import { BrowserRouter, Routes, Route } from 'react-router-dom';
import { Shell } from './components/layout/Shell';
import { DashboardView } from './components/views/dashboard/DashboardView';
import { EpisodesView } from './components/views/episodes/EpisodesView';
import { AnalysisView } from './components/views/analysis/AnalysisView';
import { CorpusView } from './components/views/corpus/CorpusView';

export default function App() {
  return (
    <BrowserRouter>
      <Shell>
        <Routes>
          <Route path="/" element={<DashboardView />} />
          <Route path="/episodes" element={<EpisodesView />} />
          <Route path="/analysis" element={<AnalysisView />} />
          <Route path="/corpus" element={<CorpusView />} />
        </Routes>
      </Shell>
    </BrowserRouter>
  );
}
