interface Props {
  tokensUsed: number;
  dailyLimit?: number;
}

export default function TokenBar({ tokensUsed, dailyLimit = 10000000 }: Props) {
  const pct = Math.min((tokensUsed / dailyLimit) * 100, 100);

  return (
    <div className="token-bar">
      <div className="token-header">
        <span className="label">Tokens</span>
        <span className="token-values">
          <span className="consumed">{formatTokens(tokensUsed)}</span>
          <span className="separator">/</span>
          <span className="remaining">{formatTokens(dailyLimit - tokensUsed)}</span>
        </span>
      </div>
      <div className="token-progress">
        <div className="token-fill" style={{ width: `${pct}%` }} />
      </div>
      <div className="token-footer">
        <span>Daily limit: {formatTokens(dailyLimit)}</span>
        <span>{pct.toFixed(1)}% used</span>
      </div>

      <style>{`
        .token-bar {
          padding: 10px 12px;
          background: #0d1f18;
          border-radius: 6px;
        }
        .token-header {
          display: flex;
          justify-content: space-between;
          margin-bottom: 6px;
        }
        .token-header .label {
          font-size: 12px;
          color: #7a9a8a;
        }
        .token-values {
          font-size: 12px;
          font-weight: 600;
        }
        .consumed { color: #a78bfa; }
        .separator { color: #4a5a4a; margin: 0 4px; }
        .remaining { color: #4ade80; }
        .token-progress {
          height: 7px;
          background: #1a3028;
          border-radius: 4px;
          overflow: hidden;
          margin-bottom: 4px;
        }
        .token-fill {
          height: 100%;
          background: linear-gradient(90deg, #8b5cf6, #6366f1);
          border-radius: 4px;
          transition: width 0.5s ease;
        }
        .token-footer {
          display: flex;
          justify-content: space-between;
          font-size: 10px;
          color: #5a7a6a;
        }
      `}</style>
    </div>
  );
}

function formatTokens(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`;
  return n.toLocaleString();
}
