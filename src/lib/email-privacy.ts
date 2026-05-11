export function maskEmailAddress(email: string) {
  const [localPart, ...domainParts] = email.split("@");
  const domain = domainParts.join("@");
  const maskedLocal = maskMiddle(localPart);

  return domain ? `${maskedLocal}@${domain}` : maskedLocal;
}

export function displayEmailAddress(email: string, emailPrivacyEnabled: boolean) {
  return emailPrivacyEnabled ? maskEmailAddress(email) : email;
}

function maskMiddle(value: string) {
  const characters = Array.from(value);

  if (characters.length <= 1) {
    return "***";
  }

  if (characters.length <= 3) {
    return `${characters[0]}***`;
  }

  const prefixLength = characters.length <= 4 ? 1 : 2;
  const prefix = characters.slice(0, prefixLength).join("");
  const suffix = characters[characters.length - 1];

  return `${prefix}***${suffix}`;
}
