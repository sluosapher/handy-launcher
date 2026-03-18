declare module 'rollup/parseAst' {
  export function parseAst(content: string, options?: unknown): { exports: string[] };
  export function parseAstAsync(content: string, options?: unknown): Promise<{ exports: string[] }>;
}

declare module 'svelte/internal' {
  export class SvelteComponent {}
}
