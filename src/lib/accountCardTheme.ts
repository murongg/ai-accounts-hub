export function getAccountCardTheme({
  isActive,
  isAlive,
}: {
  isActive: boolean;
  isAlive: boolean;
}) {
  return {
    cardClass: isActive
      ? "border-primary/20 bg-base-100/92 shadow-xl ring-1 ring-primary/10"
      : "border-base-300 bg-base-100/88 shadow-md hover:border-base-300 hover:shadow-lg",
    statusBadgeClass: isAlive
      ? "border-success/20 bg-success/10 text-success"
      : "border-error/20 bg-error/10 text-error",
    statusDotClass: isAlive ? "bg-success" : "bg-error",
    planBadgeClass: "border-primary/15 bg-primary/10 text-primary",
    secondaryBadgeClass: isActive
      ? "border-primary/15 bg-primary/10 text-primary"
      : "border-base-300 bg-base-100/80 text-base-content/70",
    primaryButtonClass: isActive
      ? "border-primary/15 bg-primary/10 text-primary hover:bg-primary/15"
      : "border-base-300 bg-base-100 text-base-content/70 hover:border-base-300 hover:bg-base-200",
  };
}

export function getQuotaProgressTone(percent: number) {
  if (percent <= 10) {
    return "text-error";
  }

  if (percent <= 30) {
    return "text-warning";
  }

  return "text-emerald-500";
}
