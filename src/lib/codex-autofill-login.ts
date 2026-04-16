export interface CodexAutofillLoginFormInput {
  email: string;
  password: string;
}

export function canShowCodexAutofillLogin(activePlatform: string) {
  return activePlatform === "codex";
}

export function normalizeCodexAutofillLoginInput(input: CodexAutofillLoginFormInput) {
  return {
    email: input.email.trim(),
    password: input.password,
  };
}
