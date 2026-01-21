/**
 * Main application layout with header, map section, and details panel
 */

import { ReactNode } from 'react';
import './AppLayout.css';

interface AppLayoutProps {
  header: ReactNode;
  mapSection: ReactNode;
  detailsPanel: ReactNode;
}

export function AppLayout({ header, mapSection, detailsPanel }: AppLayoutProps) {
  return (
    <div className="app-layout">
      <header className="app-header">{header}</header>
      <main className="main-container">
        <section className="map-section">{mapSection}</section>
        <aside className="details-panel">{detailsPanel}</aside>
      </main>
    </div>
  );
}
