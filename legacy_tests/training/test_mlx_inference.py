import numpy as np

from packages.training.mlx_inference import _as_bool_array, _as_float32_array


def test_as_float32_array_reuses_matching_input():
    arr = np.arange(8, dtype=np.float32).reshape(2, 4)

    out = _as_float32_array(arr)

    assert out.dtype == np.float32
    assert out.flags.c_contiguous
    assert np.shares_memory(out, arr)


def test_as_float32_array_converts_without_mutating_source():
    arr = np.arange(8, dtype=np.float64).reshape(2, 4)

    out = _as_float32_array(arr)

    assert out.dtype == np.float32
    assert out.flags.c_contiguous
    assert not np.shares_memory(out, arr)
    np.testing.assert_array_equal(arr, np.arange(8, dtype=np.float64).reshape(2, 4))


def test_as_bool_array_reuses_matching_input():
    arr = np.array([[True, False], [False, True]], dtype=np.bool_)

    out = _as_bool_array(arr)

    assert out.dtype == np.bool_
    assert out.flags.c_contiguous
    assert np.shares_memory(out, arr)
