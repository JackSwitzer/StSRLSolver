import numpy as np

from packages.training.inference_server import InferenceServer, _copy_obs_into


def test_copy_obs_into_pads_and_truncates():
    row = np.zeros(4, dtype=np.float32)

    _copy_obs_into(row, [1.0, 2.0], input_dim=4)
    np.testing.assert_array_equal(row, np.array([1.0, 2.0, 0.0, 0.0], dtype=np.float32))

    row.fill(0.0)
    _copy_obs_into(row, np.array([9, 8, 7, 6, 5], dtype=np.float64), input_dim=4)
    np.testing.assert_array_equal(row, np.array([9.0, 8.0, 7.0, 6.0], dtype=np.float32))


def test_forward_strategic_reuses_buffers_and_shapes_requests():
    server = InferenceServer(n_workers=1, max_batch_size=4, use_mlx=False)
    responses = []

    class DummyNet:
        input_dim = 4

    class RecordingBackend:
        def __init__(self):
            self.action_dim = 6
            self.version = 11
            self._net = DummyNet()
            self.calls = []

        def forward_batch(self, obs_batch, mask_batch):
            self.calls.append(
                {
                    "obs": obs_batch.copy(),
                    "mask": mask_batch.copy(),
                    "obs_ptr": obs_batch.__array_interface__["data"][0],
                    "mask_ptr": mask_batch.__array_interface__["data"][0],
                }
            )
            logits = np.zeros((len(obs_batch), self.action_dim), dtype=np.float32)
            values = np.arange(len(obs_batch), dtype=np.float32)
            return logits, values

    backend = RecordingBackend()
    server._strategic_backend = backend
    server._send_response = lambda worker_slot, resp: responses.append((worker_slot, resp))

    server._forward_strategic(
        [
            {
                "req_id": 1,
                "worker_slot": 0,
                "route": "strategic",
                "obs": np.array([1.0, 2.0, 3.0, 4.0], dtype=np.float32),
                "n_actions": 2,
            },
            {
                "req_id": 2,
                "worker_slot": 0,
                "route": "strategic",
                "obs": [9.0, 8.0],
                "n_actions": None,
            },
        ]
    )

    first_call = backend.calls[0]
    np.testing.assert_array_equal(
        first_call["obs"],
        np.array([[1.0, 2.0, 3.0, 4.0], [9.0, 8.0, 0.0, 0.0]], dtype=np.float32),
    )
    np.testing.assert_array_equal(
        first_call["mask"],
        np.array(
            [
                [True, True, False, False, False, False],
                [True, True, True, True, True, True],
            ],
            dtype=np.bool_,
        ),
    )
    assert [resp["req_id"] for _, resp in responses] == [1, 2]
    assert [resp["version"] for _, resp in responses] == [11, 11]

    responses.clear()
    server._forward_strategic(
        [
            {
                "req_id": 3,
                "worker_slot": 0,
                "route": "strategic",
                "obs": np.array([5, 6, 7, 8, 9], dtype=np.int64),
                "n_actions": 10,
            }
        ]
    )

    second_call = backend.calls[1]
    np.testing.assert_array_equal(
        second_call["obs"],
        np.array([[5.0, 6.0, 7.0, 8.0]], dtype=np.float32),
    )
    np.testing.assert_array_equal(
        second_call["mask"],
        np.array([[True, True, True, True, True, True]], dtype=np.bool_),
    )
    assert first_call["obs_ptr"] == second_call["obs_ptr"]
    assert first_call["mask_ptr"] == second_call["mask_ptr"]
