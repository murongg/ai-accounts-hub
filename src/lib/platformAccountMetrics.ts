export function getPlatformAccountMetrics(
  activePlatform: string,
  accounts: Array<{ is_active: boolean }>,
) {
  if (activePlatform !== "codex") {
    return {
      totalCount: 0,
      activeCount: 0,
      idleCount: 0,
    };
  }

  const activeCount = accounts.filter((account) => account.is_active).length;

  return {
    totalCount: accounts.length,
    activeCount,
    idleCount: Math.max(accounts.length - activeCount, 0),
  };
}
