const connected = document.querySelector(".connected");
const notConnected = document.querySelector(".not-connected");

chrome.storage.local.get("isConnected", (result) => {
  const isConnected = result?.isConnected || false;
  connected.classList.toggle("hidden", !isConnected);
  notConnected.classList.toggle("hidden", isConnected);
});

chrome.storage.onChanged.addListener((changes, areaName) => {
  if (areaName === "local" && changes.isConnected) {
    const isConnected = changes.isConnected.newValue;
    connected.classList.toggle("hidden", !isConnected);
    notConnected.classList.toggle("hidden", isConnected);
  }
});
