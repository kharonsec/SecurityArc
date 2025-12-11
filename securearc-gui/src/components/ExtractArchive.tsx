import React, { useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { open } from '@tauri-apps/api/dialog';
import { listen } from '@tauri-apps/api/event';

interface ProgressPayload {
    current: number;
    total: number;
    filename: string;
    status: string;
}

export function ExtractArchive() {
    const [archivePath, setArchivePath] = useState('');
    const [outputDir, setOutputDir] = useState('');
    const [password, setPassword] = useState('');
    const [loading, setLoading] = useState(false);
    const [status, setStatus] = useState('');
    const [error, setError] = useState<string | null>(null);
    const [progress, setProgress] = useState<ProgressPayload | null>(null);

    const handleSelectArchive = async () => {
        try {
            const selected = await open({
                filters: [{ name: 'SecureArc Archive', extensions: ['sarc'] }]
            });
            if (selected) {
                setArchivePath(selected as string);
            }
        } catch (err) {
            console.error(err);
        }
    };

    const handleSelectOutput = async () => {
        try {
            const selected = await open({
                directory: true,
            });
            if (selected) {
                setOutputDir(selected as string);
            }
        } catch (err) {
            console.error(err);
        }
    };

    const handleExtract = async () => {
        if (!archivePath || !outputDir || !password) {
            setError('Please fill all fields');
            return;
        }

        setLoading(true);
        setError(null);
        setStatus('Starting extraction...');
        setProgress({ current: 0, total: 0, filename: '', status: 'Starting' });

        const unlisten = await listen<ProgressPayload>('extract-progress', (event) => {
            setProgress(event.payload);
        });

        try {
            await invoke('extract_archive', {
                request: {
                    archivePath,
                    outputPath: outputDir,
                    password
                }
            });
            setStatus('Extraction complete!');
            setPassword('');
        } catch (err) {
            setError(err as string);
            setStatus('');
        } finally {
            unlisten();
            setLoading(false);
            setProgress(null);
        }
    };

    return (
        <div className="tab-content">
            <h2>Extract Archive</h2>

            <div className="form-group">
                <label>Archive File</label>
                <div className="input-row">
                    <input type="text" value={archivePath} readOnly placeholder="Select archive..." />
                    <button onClick={handleSelectArchive} disabled={loading}>Browse</button>
                </div>
            </div>

            <div className="form-group">
                <label>Output Directory</label>
                <div className="input-row">
                    <input type="text" value={outputDir} readOnly placeholder="Select destination..." />
                    <button onClick={handleSelectOutput} disabled={loading}>Browse</button>
                </div>
            </div>

            <div className="form-group">
                <label>Password</label>
                <input
                    type="password"
                    value={password}
                    onChange={e => setPassword(e.target.value)}
                    placeholder="Enter password"
                    disabled={loading}
                />
            </div>

            {error && <div className="error-msg">{error}</div>}
            {status && <div className="success-msg">{status}</div>}

            {loading && progress && (
                <div className="progress-container">
                    <div className="progress-info">
                        <span>{progress.status}: {progress.filename}</span>
                        <span>{progress.current} / {progress.total}</span>
                    </div>
                    <progress value={progress.current} max={progress.total}></progress>
                </div>
            )}

            <button className="action-btn" onClick={handleExtract} disabled={loading}>
                {loading ? 'Extracting...' : 'Extract Files'}
            </button>
        </div>
    );
}
