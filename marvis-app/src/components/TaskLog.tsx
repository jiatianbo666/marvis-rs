import { useEffect, useRef } from "react";

interface Props {
  entries: string[];
}

export default function TaskLog({ entries }: Props) {
  const bottomRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [entries]);

  return (
    <div className="task-log">
      <div className="task-log-header">📝 Activity Log</div>
      {entries.length === 0 && (
        <div className="entry placeholder">Waiting for tasks...</div>
      )}
      {entries.map((entry, i) => (
        <div key={i} className="entry">
          {entry}
        </div>
      ))}
      <div ref={bottomRef} />

      <style>{`
        .task-log {
          flex: 1;
          overflow-y: auto;
          min-height: 0;
          background: #0a1812;
          border-radius: 6px;
          padding: 8px;
        }
        .task-log-header {
          font-size: 11px;
          font-weight: 600;
          color: #5a7a6a;
          margin-bottom: 6px;
          padding-bottom: 4px;
          border-bottom: 1px solid #152a20;
        }
        .entry {
          padding: 4px 6px;
          font-size: 11px;
          color: #8a9a8a;
          border-bottom: 1px solid #0d1f16;
          word-break: break-word;
          line-height: 1.4;
        }
        .entry:last-child { border-bottom: none; }
        .entry.placeholder { color: #3a5a4a; font-style: italic; }
      `}</style>
    </div>
  );
}
