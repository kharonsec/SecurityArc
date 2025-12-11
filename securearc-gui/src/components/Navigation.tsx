import React from 'react';

interface NavigationProps {
    activeTab: string;
    onTabChange: (tab: string) => void;
}

export function Navigation({ activeTab, onTabChange }: NavigationProps) {
    return (
        <nav className="nav-bar">
            <button
                className={`nav-item ${activeTab === 'create' ? 'active' : ''}`}
                onClick={() => onTabChange('create')}
            >
                Create Archive
            </button>
            <button
                className={`nav-item ${activeTab === 'extract' ? 'active' : ''}`}
                onClick={() => onTabChange('extract')}
            >
                Extract Archive
            </button>
            <button
                className={`nav-item ${activeTab === 'view' ? 'active' : ''}`}
                onClick={() => onTabChange('view')}
            >
                View / Info
            </button>
        </nav>
    );
}
