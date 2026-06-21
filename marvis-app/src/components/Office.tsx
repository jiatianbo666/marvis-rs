import { useMemo } from "react";
import { AgentState, DESK_SPOTS, PUBLIC_SPOTS, ZONE_LAYOUT, MAX_ZONE_OCCUPANCY, IdleVariant } from "../data/agents";
import AgentAvatar from "./AgentAvatar";
import DeskComputer from "./DeskComputer";
import ZoneIcon from "./ZoneIcon";

interface Props { agents: Map<string, AgentState>; idleVariants: Map<string, IdleVariant> }

export default function Office({ agents, idleVariants }: Props) {
  const agentList = Array.from(agents.values());

  const zoneCounts = useMemo(() => {
    const counts: Record<string, number> = { coffee: 0, workout: 0, bathroom: 0 };
    idleVariants.forEach((v) => { if (v in counts) counts[v]++; });
    return counts;
  }, [idleVariants]);

  const positions = useMemo(() => {
    const pos = new Map<string, { x: number; y: number }>();
    const zoneSlots: Record<string, number> = {};
    agentList.forEach((agent) => {
      const variant = idleVariants.get(agent.id) || "sleeping";
      if (agent.status === "idle" && variant !== "sleeping" && variant !== "wandering") {
        const spot = PUBLIC_SPOTS[variant] || DESK_SPOTS[agent.id] || { x: 50, y: 50 };
        const layout = ZONE_LAYOUT[variant] || { perRow: 1, xGap: 0, yGap: 0 };
        const idx = zoneSlots[variant] || 0;
        zoneSlots[variant] = idx + 1;
        const row = Math.floor(idx / layout.perRow);
        // First animal offset -10, second +18
        const colOff = idx === 0 ? -3 : 12;
        pos.set(agent.id, { x: spot.x + colOff, y: spot.y + row * layout.yGap + 2 });
      } else {
        pos.set(agent.id, DESK_SPOTS[agent.id] || { x: 50, y: 50 });
      }
    });
    return pos;
  }, [agentList, idleVariants]);

  return (
    <div className="office-scene">
      <div className="office-floor" />

      {/* Public zones */}
      <div className="public-zone" style={{ left: "3%", top: "6%", width: "22%", height: "26%" }}>
        <ZoneIcon zone="coffee" size={100} />
        <div className="zone-label">Coffee</div>
        <div className="zone-count">{zoneCounts.coffee || 0}/2</div>
      </div>
      <div className="public-zone" style={{ left: "3%", top: "37%", width: "22%", height: "26%" }}>
        <ZoneIcon zone="gym" size={125} />
        <div className="zone-label">Gym</div>
        <div className="zone-count">{zoneCounts.workout || 0}/2</div>
      </div>
      <div className="public-zone" style={{ left: "3%", top: "68%", width: "22%", height: "26%" }}>
        <ZoneIcon zone="rest" size={42} />
        <div className="zone-label">Rest</div>
        <div className="zone-count">{zoneCounts.bathroom || 0}/2</div>
      </div>

      {/* Desks */}
      {Object.entries(DESK_SPOTS).map(([id, spot]) => (
        <div key={id} className="desk" style={{ left: `${spot.x}%`, top: `${spot.y}%` }}>
          <div className="desk-screen"><DeskComputer /></div>
          <div className="desk-table" />
          <span className="desk-label">{id.replace(/_/g, " ")}</span>
        </div>
      ))}

      {/* Agents */}
      <div className="avatars-layer">
        {agentList.map((agent) => {
          const pos = positions.get(agent.id) || { x: 50, y: 50 };
          return (
            <div key={agent.id} className="pony-node" style={{ left: `${pos.x}%`, top: `${pos.y}%` }}>
              <AgentAvatar agent={agent} size={agent.id === "browser_agent" ? 100 : 74} idleVariant={idleVariants.get(agent.id) || "sleeping"} />
            </div>
          );
        })}
      </div>

      <style>{`
        .office-scene { position: relative; width: 100%; height: 100%; border-radius: 20px; background: linear-gradient(180deg,#f0f2f0,#e3e5e3 60%,#dadcda); border: 1px solid #d0d0d0; overflow: hidden; }
        .office-floor { position: absolute; inset: 0; background: repeating-linear-gradient(0deg,transparent,transparent 49px,#d8dad8 49px,#d8dad8 50px); opacity: 0.4; }
        .public-zone { position: absolute; border-radius: 18px; background: linear-gradient(180deg,rgba(255,255,255,0.7),rgba(240,240,240,0.9)); border: 1px solid #dadada; box-shadow: inset 0 -18px 25px rgba(0,0,0,0.03); padding: 12px; display: flex; flex-direction: column; align-items: center; justify-content: center; }
        .public-zone .zone-label { position: absolute; bottom: 26px; left: 50%; transform: translateX(-50%); font-size: 13px; font-weight: 700; color: #555; white-space: nowrap; }
        .public-zone .zone-count { position: absolute; bottom: 8px; left: 50%; transform: translateX(-50%); font-size: 10px; color: #999; }
        .desk { position: absolute; width: 100px; height: 120px; transform: translate(-50%,-50%); display: flex; flex-direction: column; align-items: center; gap: 2px; }
        .desk-screen { width: 88px; height: 66px; background: transparent; border-radius: 4px 4px 0 0; display: flex; align-items: center; justify-content: center; }
        .desk-table { width: 75%; height: 40%; background: linear-gradient(180deg,#e0d8c8,#c8c0b0); border-radius: 3px; box-shadow: 0 2px 6px rgba(0,0,0,0.15); }
        .desk-label { font-size: 11px; color: #555; text-transform: capitalize; margin-top: 6px; white-space: nowrap; font-weight: 700; position: relative; z-index: 20; text-shadow: 0 1px 0 rgba(255,255,255,0.8); letter-spacing: 0.5px; }
        .avatars-layer { position: absolute; inset: 0; pointer-events: none; }
      `}</style>
    </div>
  );
}
