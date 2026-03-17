import { Sparkline, type SparklineMarker } from './Sparkline';

interface MetricsChartsProps {
  floorHistory: number[];
  lossHistory: number[];
  trainStepMarkers?: { index: number; step: number }[];
}

export const MetricsCharts = ({ floorHistory, lossHistory, trainStepMarkers }: MetricsChartsProps) => {
  const markers: SparklineMarker[] = (trainStepMarkers ?? []).map((m) => ({
    index: m.index,
    label: `T${m.step}`,
  }));

  return (
    <div style={{ display: 'flex', gap: 16, flexWrap: 'wrap' }}>
      <Sparkline
        data={floorHistory}
        width={280}
        height={60}
        color="#00ff41"
        label="Avg Floor"
        markers={markers}
      />
      <Sparkline
        data={lossHistory}
        width={280}
        height={60}
        color="#f0883e"
        label="Loss"
        markers={markers}
      />
    </div>
  );
};
