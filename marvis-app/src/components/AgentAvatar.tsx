import { useEffect, useRef, useMemo } from "react";
import { AgentState, IdleVariant } from "../data/agents";
import pandaAnim from "../assets/lottie/pets/panda.json";
import foxAnim from "../assets/lottie/pets/fox.json";
import horseAnim from "../assets/lottie/pets/horse.json";
import rabbitAnim from "../assets/lottie/pets/rabbit.json";
import dogAnim from "../assets/lottie/pets/dog.json";
import pigAnim from "../assets/lottie/pets/pig.json";

const PETS: Record<string, object> = { panda: pandaAnim, fox: foxAnim, horse: horseAnim, rabbit: rabbitAnim, dog: dogAnim, pig: pigAnim };

interface Props { agent: AgentState; size?: number; idleVariant?: IdleVariant }

export default function AgentAvatar({ agent, size = 72, idleVariant: _v }: Props) {
  const containerRef = useRef<HTMLDivElement>(null);
  const animRef = useRef<any>(null);

  const petTheme = agent.id.includes("file") ? "fox"
    : agent.id.includes("computer") ? "horse"
    : agent.id.includes("app") ? "rabbit"
    : agent.id.includes("browser") ? "dog"
    : agent.id.includes("search") ? "pig"
    : "panda";

  const animData = PETS[petTheme] || pandaAnim;
  const isActive = agent.status === "working" || agent.status === "thinking" || agent.status === "dispatching";
  const isDone = agent.status === "done";
  const isError = agent.status === "error";

  useEffect(() => {
    const el = containerRef.current;
    if (!el) return;
    let cancelled = false;

    import("lottie-web/build/player/lottie_svg.min.js").then((m: any) => {
      if (cancelled || !containerRef.current) return;
      const lot = m.default || m;
      animRef.current?.destroy();
      animRef.current = lot.loadAnimation({
        container: containerRef.current,
        renderer: "svg",
        loop: true,
        autoplay: true,
        animationData: animData,
        rendererSettings: { preserveAspectRatio: "xMidYMid meet" },
      });
    }).catch(() => {});

    return () => { cancelled = true; animRef.current?.destroy(); animRef.current = null; };
  }, [animData]);

  return (
    <div className={`pony-avatar state-${agent.status}`} title={agent.message}>
      {/* Active glow ring */}
      {isActive && <div className="active-ring" />}
      {isDone && <div className="done-ring" />}
      {isError && <div className="error-ring" />}

      <div ref={containerRef} style={{ width: size, height: size }} />

      <div className="agent-shadow" style={{ width: size * 0.5, height: size * 0.12 }} />
    </div>
  );
}
