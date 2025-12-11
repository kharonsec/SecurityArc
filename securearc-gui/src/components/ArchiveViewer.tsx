import React, { useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { open } from '@tauri-apps/api/dialog';

interface ArchiveInfo {
    max_attempts: number;
    current_attempts: number;
    remaining_attempts: number;
    destroyed: boolean;
    file_count: number;
    files?: string[];
}

interface ArchiveViewerProps {
    initialPath?: string;
}

export function ArchiveViewer({ initialPath }: ArchiveViewerProps) {
    const [archivePath, setArchivePath] = useState(initialPath || '');
    const [password, setPassword] = useState('');
    const [info, setInfo] = useState<ArchiveInfo | null>(null);
    const [fileList, setFileList] = useState<string[]>([]);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);

    // Auto-load info if path is provided
    React.useEffect(() => {
        if (initialPath) {
            handleGetInfo();
        }
    }, [initialPath]);

    const handleSelectArchive = async () => {
        try {
            const selected = await open({
                filters: [{ name: 'SecureArc Archive', extensions: ['sarc'] }]
            });
            if (selected) {
                setArchivePath(selected as string);
                // Reset state
                setInfo(null);
                setFileList([]);
                setError(null);
            }
        } catch (err) {
            console.error(err);
        }
    };

    const handleGetInfo = async () => {
        if (!archivePath) return;
        setLoading(true);
        setError(null);
        try {
            const result = await invoke<ArchiveInfo>('get_archive_info', { archive_path: archivePath });
            setInfo(result);
        } catch (err) {
            setError(err as string);
        } finally {
            setLoading(false);
        }
    };

    const handleListFiles = async () => {
        if (!archivePath || !password) {
            setError('Password required to list files');
            return;
        }
        setLoading(true);
        setError(null);
        try {
            // Need a backend command that returns file list independently or use existing logic
            // Note: Backend 'list_archive' struct might differ, checking main.rs...
            // Assuming list_archive returns ArchiveInfo with files populated or similar
            // Actually main.rs signature is `list_archive(request: ListRequest) -> Result<ArchiveInfo, ...>`
            // where ArchiveInfo has `files` field.

            const result = await invoke<ArchiveInfo>('list_archive', {
                request: {
                    archive_path: archivePath,
                    password
                }
            });
            setInfo(result);
            if (result.files) {
                setFileList(result.files);
            }
        } catch (err) {
            setError(err as string);
        } finally {
            setLoading(false);
        }
    };

    return (
        <div className="tab-content">
            <h2>Archive Viewer</h2>

            <div className="form-group">
                <label>Archive File</label>
                <div className="input-row">
                    <input type="text" value={archivePath} readOnly placeholder="Select archive..." />
                    <button onClick={handleSelectArchive} disabled={loading}>Browse</button>
                </div>
            </div>

            <div className="action-row">
                <button onClick={handleGetInfo} disabled={!archivePath || loading}>
                    Get Info
                </button>
            </div>

            {info && (
                <div className="info-panel">
                    <h3>Metadata</h3>
                    <div className="info-grid">
                        <div className="info-item">
                            <span className="label">Status:</span>
                            <span className={`value ${info.destroyed ? 'danger' : 'success'}`}>
                                {info.destroyed ? 'DESTROYED' : 'Active'}
                            </span>
                        </div>
                        <div className="info-item">
                            <span className="label">Remaining Attempts:</span>
                            <span className={`value ${info.remaining_attempts <= 2 ? 'warning' : ''}`}>
                                {info.remaining_attempts} / {info.max_attempts}
                            </span>
                        </div>
                        <div className="info-item">
                            <span className="label">File Count:</span>
                            <span className="value">{info.file_count}</span>
                        </div>
                    </div>
                </div>
            )}

            <div className="divider"></div>

            <div className="form-group">
                <label>Unlock to View Contents</label>
                <div className="input-row">
                    <input
                        type="password"
                        value={password}
                        onChange={e => setPassword(e.target.value)}
                        placeholder="Enter password..."
                        disabled={loading || info?.destroyed}
                    />
                    <button onClick={handleListFiles} disabled={loading || !password || info?.destroyed}>
                        Unlock & List
                    </button>
                </div>
            </div>

            {error && <div className="error-msg">{error}</div>}

            {fileList.length > 0 && (
                <div className="file-list-container">
                    <h3>Contents</h3>
                    <ul className="file-list-view">
                        {fileList.map((f, i) => (
                            <li key={i} className="file-list-item">
                                <span className="file-icon">📄</span>
                                <span className="file-name">{f}</span>
                            </li>
                        ))}
                    </ul>
                </div>
            )}
        </div>
    );
}
