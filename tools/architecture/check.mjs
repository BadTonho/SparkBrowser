import { readdirSync, readFileSync } from 'node:fs';
import { dirname, resolve, relative } from 'node:path';
import { pathToFileURL } from 'node:url';
import ts from 'typescript';

function files(directory, extensions) {
  return readdirSync(directory, { withFileTypes: true }).flatMap(entry => {
    const path = resolve(directory, entry.name);
    return entry.isDirectory() ? files(path, extensions) : extensions.some(ext => path.endsWith(ext)) ? [path] : [];
  });
}

export function findCycles(graph) {
  const active = new Set(), visited = new Set(), stack = [], cycles = [];
  function visit(node) {
    if (active.has(node)) { cycles.push([...stack.slice(stack.indexOf(node)), node]); return; }
    if (visited.has(node)) return;
    active.add(node); stack.push(node);
    for (const target of graph.get(node) ?? []) visit(target);
    stack.pop(); active.delete(node); visited.add(node);
  }
  for (const node of graph.keys()) visit(node);
  return cycles;
}

export function checkArchitecture() {
  const graph = new Map();
  const errors = [];
  const label = path => relative(resolve('.'), path).replaceAll('\\', '/');
  const sourceFiles = files('src/lib', ['.ts', '.js']);
  for (const path of files('src/lib', ['.ts', '.js', '.svelte'])) {
    const text = readFileSync(path, 'utf8');
    const code = path.endsWith('.svelte') ? [...text.matchAll(/<script\b[^>]*>([\s\S]*?)<\/script>/g)].map(match => match[1]).join('\n') : text;
    const source = ts.createSourceFile(path, code, ts.ScriptTarget.Latest, true);
    const dependencies = [];
    for (const statement of source.statements) {
      if (!ts.isImportDeclaration(statement) && !ts.isExportDeclaration(statement)) continue;
      if (statement.isTypeOnly || statement.importClause?.isTypeOnly) continue;
      const bindings = statement.importClause?.namedBindings;
      if (!statement.importClause?.name && bindings && ts.isNamedImports(bindings) && bindings.elements.every(item => item.isTypeOnly)) continue;
      const specifier = statement.moduleSpecifier?.text;
      if (!specifier?.startsWith('.')) continue;
      const base = resolve(dirname(path), specifier);
      const target = [base, base + '.ts', base + '.js', resolve(base, 'index.ts')].find(candidate => sourceFiles.includes(candidate));
      if (!target) continue;
      dependencies.push(target);
      const from = label(path), to = label(target);
      if (/stores\/browser\/(state|address|presentation)\.ts$/.test(to) && !from.includes('/stores/browser/')) {
        errors.push(`${from} imports private browser state: ${to}`);
      }
      if (bindings && ts.isNamedImports(bindings)) {
        for (const item of bindings.elements) {
          const imported = (item.propertyName ?? item.name).text;
          const allowed = {
            applyConfirmedSplit: ['src/lib/stores/browser/actions.ts'],
            confirmWorkspaceSelection: ['src/lib/stores/browser/actions.ts', 'src/lib/stores/workspaceStore.ts'],
            applyWorkspaceSnapshot: ['src/lib/stores/backupStore.ts'],
          }[imported];
          if (allowed && !allowed.includes(from)) errors.push(`${from} imports a private session mutation: ${imported}`);
        }
      }
    }
    if (!path.endsWith('.svelte')) graph.set(path, dependencies);
  }
  for (const cycle of findCycles(graph)) errors.push(`Runtime import cycle: ${cycle.map(label).join(' -> ')}`);
  for (const path of files('src-tauri/src', ['.rs'])) {
    const source = readFileSync(path, 'utf8'), name = label(path);
    if (!name.startsWith('src-tauri/src/state/') && /\.edit_(?:tabs|workspaces|active_tab_id|active_workspace_id|split_layouts|next_tab_counter)\s*\(/.test(source)) {
      errors.push(`${name} bypasses session operations`);
    }
    if (!name.startsWith('src-tauri/src/state/') && /\.(?:select_tab|move_tab)\s*\(/.test(source)) errors.push(`${name} bypasses committed session selection/movement`);
    if (!name.startsWith('src-tauri/src/state/') && /(?:crate::state|super)::domains::/.test(source)) errors.push(`${name} imports private state storage`);
    if (name !== 'src-tauri/src/window/content_lifecycle.rs' && /content_factory::/.test(source)) errors.push(`${name} bypasses content lifecycle`);
    if (!['src-tauri/src/window/content_lifecycle.rs', 'src-tauri/src/state/domains/content_bootstrap.rs'].includes(name) && /\.navigate\s*\(/.test(source)) {
      errors.push(`${name} dispatches native navigation outside content lifecycle`);
    }
  }
  if (errors.length) throw new Error(errors.join('\n'));
  console.log(`Architecture checked: ${graph.size} frontend modules; private state and content lifecycle boundaries.`);
}

if (process.argv[1] && pathToFileURL(resolve(process.argv[1])).href === import.meta.url) checkArchitecture();
