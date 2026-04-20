import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";

const accountsPage = readFileSync(path.resolve("src/pages/accounts-page.tsx"), "utf8");
const accountsWorkspace = readFileSync(path.resolve("src/containers/accounts-workspace.tsx"), "utf8");
const accountCard = readFileSync(path.resolve("src/components/account-card.tsx"), "utf8");
const accountListItem = readFileSync(path.resolve("src/components/account-list-item.tsx"), "utf8");
const codexAccounts = readFileSync(path.resolve("src/lib/codex-accounts.ts"), "utf8");
const codexType = readFileSync(path.resolve("src/types/codex.ts"), "utf8");
const i18nSource = readFileSync(path.resolve("src/lib/i18n.ts"), "utf8");

test("codex accounts page wires single-account refresh controls", () => {
  assert.ok(accountsPage.includes("onRefreshAccount"));
  assert.ok(accountsPage.includes("refreshingAccountId"));
  assert.ok(accountsPage.includes('activePlatform === "codex"'));
  assert.ok(accountsPage.includes("onRefreshClick={onRefreshAccount}"));
  assert.ok(accountsPage.includes("isRefreshing={refreshingAccountId === account.id}"));

  assert.ok(accountsWorkspace.includes("refreshingCodexAccountId"));
  assert.ok(accountsWorkspace.includes("handleRefreshCodexAccount"));

  assert.ok(accountCard.includes("onRefreshClick"));
  assert.ok(accountCard.includes("refreshDisabled"));
  assert.ok(accountCard.includes("copy.accounts.refreshAccountAria"));

  assert.ok(accountListItem.includes("onRefreshClick"));
  assert.ok(accountListItem.includes("refreshDisabled"));
  assert.ok(accountListItem.includes("copy.accounts.refreshAccountAria"));
});

test("codex single-account refresh adds command wrapper and accelerated metadata", () => {
  assert.ok(codexAccounts.includes('invoke<void>("refresh_codex_account_usage", { accountId })'));
  assert.ok(codexType.includes("refresh_accelerated_until: string | null;"));
  assert.ok(i18nSource.includes("refreshAccountAria"));
  assert.ok(i18nSource.includes("acceleratedRefreshActive"));
  assert.ok(accountsPage.includes("codexAccount.refresh_accelerated_until"));
  assert.ok(accountsPage.includes("copy.accounts.acceleratedRefreshActive"));
});
