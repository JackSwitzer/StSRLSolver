/**
 * Main App component
 * Combines all components into the Seed Oracle application
 */

import { AppLayout } from './components/Layout/AppLayout';
import { SeedInput } from './components/SeedInput';
import { MapCanvas, ActTabs, NeowSection } from './components/Map';
import { FloorDetails } from './components/FloorDetails';
import './styles/globals.css';

function Header() {
  return (
    <>
      <div className="logo">
        SEED <span>ORACLE</span>
      </div>
      <SeedInput />
    </>
  );
}

function MapSection() {
  return (
    <>
      <NeowSection />
      <ActTabs />
      <MapCanvas />
    </>
  );
}

function DetailsPanel() {
  return <FloorDetails />;
}

export default function App() {
  return (
    <AppLayout
      header={<Header />}
      mapSection={<MapSection />}
      detailsPanel={<DetailsPanel />}
    />
  );
}
