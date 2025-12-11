import React, { useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { listen, Event } from '@tauri-apps/api/event';
import { Navigation } from './components/Navigation';
import { CreateArchive } from './components/CreateArchive';
import { ExtractArchive } from './components/ExtractArchive';
import { ArchiveViewer } from './components/ArchiveViewer';
import './App.css';

function App() {
  const [activeTab, setActiveTab] = useState('create');
  const [viewerPath, setViewerPath] = useState('');

  React.useEffect(() => {
    const unlisten = listen('open-file', (event: Event<string>) => {
      const path = event.payload;
      setViewerPath(path);
      setActiveTab('view');
    });
    return () => {
      unlisten.then(f => f());
    };
  }, []);

  const renderContent = () => {
    switch (activeTab) {
      case 'create':
        return <CreateArchive />;
      case 'extract':
        return <ExtractArchive />;
      case 'view':
        return <ArchiveViewer initialPath={viewerPath} />;
      default:
        return <CreateArchive />;
    }
  };

  return (
    <div className="app-container">
      <header className="app-header">
        <h1>SecureArc</h1>
        <div className="subtitle">Self-Destructing Encrypted Storage</div>
      </header>

      <Navigation activeTab={activeTab} onTabChange={setActiveTab} />

      <main className="main-content">
        {renderContent()}
      </main>
    </div>
  );
}

export default App;
