export const criticalScenarios = Object.freeze([
  'backup-identities', 'persistence-split', 'persistence-workspaces',
  'workspace-switcher', 'recovery', 'shutdown', 'security-boundaries',
]);

export function selectScenarios(args, scenarios) {
  if (!args.length) return Object.keys(scenarios);
  if (args.length === 2 && args[0] === '--suite' && args[1] === 'critical') return [...criticalScenarios];
  if (args.length === 2 && args[0] === '--case' && Object.hasOwn(scenarios, args[1])) return [args[1]];
  throw new Error('Use --list, --case <scenario> or --suite critical.');
}
