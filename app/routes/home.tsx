import type { Route } from "./+types/home";
import { Game } from "../features/game/components/Game";

export function meta({}: Route.MetaArgs) {
  return [
    { title: "2048 — Join the numbers" },
    {
      name: "description",
      content: "A polished, keyboard and touch-friendly version of 2048.",
    },
  ];
}

export default function Home() {
  return <Game />;
}
