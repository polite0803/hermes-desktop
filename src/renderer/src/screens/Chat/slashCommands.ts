export interface SlashCommand {
  name: string;
  descriptionKey: string;
  category: "chat" | "agent" | "tools" | "info";
  local?: boolean;
}

export const SLASH_COMMANDS: SlashCommand[] = [
  {
    name: "/new",
    descriptionKey: "chat.commands.new",
    category: "chat",
    local: true,
  },
  {
    name: "/clear",
    descriptionKey: "chat.commands.clear",
    category: "chat",
    local: true,
  },
  { name: "/btw", descriptionKey: "chat.commands.btw", category: "agent" },
  {
    name: "/approve",
    descriptionKey: "chat.commands.approve",
    category: "agent",
  },
  { name: "/deny", descriptionKey: "chat.commands.deny", category: "agent" },
  {
    name: "/status",
    descriptionKey: "chat.commands.status",
    category: "agent",
  },
  { name: "/reset", descriptionKey: "chat.commands.reset", category: "agent" },
  {
    name: "/compact",
    descriptionKey: "chat.commands.compact",
    category: "agent",
  },
  { name: "/undo", descriptionKey: "chat.commands.undo", category: "agent" },
  { name: "/retry", descriptionKey: "chat.commands.retry", category: "agent" },
  {
    name: "/fast",
    descriptionKey: "chat.commands.fast",
    category: "agent",
    local: true,
  },
  {
    name: "/compress",
    descriptionKey: "chat.commands.compress",
    category: "agent",
  },
  {
    name: "/usage",
    descriptionKey: "chat.commands.usage",
    category: "agent",
    local: true,
  },
  { name: "/debug", descriptionKey: "chat.commands.debug", category: "agent" },
  { name: "/goal", descriptionKey: "chat.commands.goal", category: "agent" },
  { name: "/steer", descriptionKey: "chat.commands.steer", category: "agent" },
  { name: "/queue", descriptionKey: "chat.commands.queue", category: "agent" },
  {
    name: "/update",
    descriptionKey: "chat.commands.update",
    category: "agent",
  },
  { name: "/web", descriptionKey: "chat.commands.web", category: "tools" },
  { name: "/image", descriptionKey: "chat.commands.image", category: "tools" },
  {
    name: "/browse",
    descriptionKey: "chat.commands.browse",
    category: "tools",
  },
  { name: "/code", descriptionKey: "chat.commands.code", category: "tools" },
  { name: "/file", descriptionKey: "chat.commands.file", category: "tools" },
  { name: "/shell", descriptionKey: "chat.commands.shell", category: "tools" },
  { name: "/help", descriptionKey: "chat.commands.help", category: "info" },
  { name: "/tools", descriptionKey: "chat.commands.tools", category: "info" },
  { name: "/skills", descriptionKey: "chat.commands.skills", category: "info" },
  {
    name: "/reload-skills",
    descriptionKey: "chat.commands.reloadSkills",
    category: "info",
  },
  { name: "/kanban", descriptionKey: "chat.commands.kanban", category: "info" },
  {
    name: "/curator",
    descriptionKey: "chat.commands.curator",
    category: "info",
  },
  { name: "/model", descriptionKey: "chat.commands.model", category: "info" },
  { name: "/memory", descriptionKey: "chat.commands.memory", category: "info" },
  {
    name: "/persona",
    descriptionKey: "chat.commands.persona",
    category: "info",
  },
  {
    name: "/version",
    descriptionKey: "chat.commands.version",
    category: "info",
  },
];
