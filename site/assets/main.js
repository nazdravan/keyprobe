// The main button offers the file for the visitor's system; without JavaScript it leads to the
// downloads table. Phones, tablets and Chromebooks get no pick: KeyProbe's packages are for
// desktops (the browser version is one click away).

export function pickDownload(ua, platform, maxTouchPoints = 0) {
  if (/Android|iPhone|iPad|iPod|CrOS/i.test(ua)) return null;
  if (/Win/i.test(platform) || /Windows/i.test(ua)) return "windows";
  // iPadOS reports a Mac; a touch screen gives it away.
  if (/Mac/i.test(platform)) return maxTouchPoints > 1 ? null : "mac";
  if (/Linux/i.test(platform) || /X11/.test(ua)) return "linux";
  return null;
}

function setupDownload() {
  const button = document.getElementById("download-main");
  if (!button) return;
  const system = pickDownload(navigator.userAgent, navigator.platform || "", navigator.maxTouchPoints || 0);
  if (!system) return;
  let kinds = {};
  try {
    kinds = JSON.parse(button.dataset.kinds || "{}");
  } catch {
    return;
  }
  const url = kinds[system];
  const label = button.getAttribute(`data-label-${system}`);
  if (!url || !label) return;
  button.href = url;
  button.textContent = label;
}

if (typeof document !== "undefined") {
  setupDownload();
}
