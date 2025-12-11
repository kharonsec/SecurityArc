import React, { useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { open, save } from '@tauri-apps/api/dialog';
import { listen } from '@tauri-apps/api/event';

interface ProgressPayload {
    current: number;
    total: number;
    filename: string;
    status: string;
}

export function CreateArchive() {
    const [files, setFiles] = useState<string[]>([]);
    const [output, setOutput] = useState('');
    const [password, setPassword] = useState('');
    const [confirmPassword, setConfirmPassword] = useState('');
    const [loading, setLoading] = useState(false);
    const [status, setStatus] = useState('');
    const [error, setError] = useState<string | null>(null);
    const [progress, setProgress] = useState<ProgressPayload | null>(null);

    const handleAddFiles = async () => {
        try {
            const selected = await open({
                multiple: true,
                filters: [{ name: 'All Files', extensions: ['*'] }]
            });
            if (selected) {
                const newFiles = Array.isArray(selected) ? selected : [selected];
                setFiles(prev => [...prev, ...newFiles]);

                // Default output to source location + .sarc if not set
                if (!output && newFiles.length > 0) {
                    setOutput(newFiles[0] + '.sarc');
                }
            }
        } catch (err) {
            console.error(err);
        }
    };

    const handleAddFolders = async () => {
        try {
            const selected = await open({
                multiple: true,
                directory: true,
            });
            if (selected) {
                const newFolders = Array.isArray(selected) ? selected : [selected];
                setFiles(prev => [...prev, ...newFolders]);

                // Default output to source location + .sarc if not set
                if (!output && newFolders.length > 0) {
                    setOutput(newFolders[0] + '.sarc');
                }
            }
        } catch (err) {
            console.error(err);
        }
    };

    const handleSelectOutput = async () => {
        try {
            const selected = await save({
                filters: [{ name: 'SecureArc Archive', extensions: ['sarc'] }]
            });
            if (selected) {
                setOutput(selected as string);
            }
        } catch (err) {
            console.error(err);
        }
    };

    const handleCreate = async () => {
        if (files.length === 0 || !output || !password) {
            setError('Please fill all fields');
            return;
        }
        if (password !== confirmPassword) {
            setError('Passwords do not match');
            return;
        }

        setLoading(true);
        setError(null);
        setStatus('Starting process...');
        setProgress({ current: 0, total: files.length, filename: '', status: 'Starting' });

        // Listen for progress
        const unlisten = await listen<ProgressPayload>('create-progress', (event) => {
            setProgress(event.payload);
        });

        try {
            // Note: Backend command needs to match main.rs definition
            await invoke('create_archive', {
                request: {
                    outputPath: output,
                    files,
                    password,
                    // Default options for now
                    maxAttempts: 5,
                    encryption: "aes256",
                    compression: "lzma2"
                }
            });
            setStatus('Archive created successfully!');
            setFiles([]);
            setOutput('');
            setPassword('');
            setConfirmPassword('');
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
            <h2>Create New Archive</h2>

            <div className="form-group">
                <label>Input Files</label>
                <div className="file-list">
                    {files.map((f, i) => <div key={i} className="file-item">{f}</div>)}
                </div>
                <div className="button-group">
                    <button onClick={handleAddFiles} disabled={loading}>+ Add Files</button>
                    <button onClick={handleAddFolders} disabled={loading} style={{ marginLeft: '10px' }}>+ Add Folder</button>
                </div>
            </div>

            <div className="form-group">
                <label>Output Archive</label>
                <div className="input-row">
                    <input type="text" value={output} readOnly placeholder="Select output path..." />
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
                <input
                    type="password"
                    value={confirmPassword}
                    onChange={e => setConfirmPassword(e.target.value)}
                    placeholder="Confirm password"
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

            <button className="action-btn" onClick={handleCreate} disabled={loading}>
                {loading ? 'Processing...' : 'Create Archive'}
            </button>
        </div>
    );
}
