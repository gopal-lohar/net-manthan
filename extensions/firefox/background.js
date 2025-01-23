let isConnected = false;

function updateState(newState) {
  isConnected = newState;
  chrome.storage.local.set({ isConnected: isConnected });
}

let port;
try {
  port = browser.runtime.connectNative("com.net.manthan");
} catch (e) {
  console.error(e);
}

if (port) {
  updateState(true);

  port.onDisconnect.addListener((p) => {
    if (p.error) {
      console.error(`Disconnected due to error: ${p.error.message}`);
      updateState(false);
    }
  });

  browser.downloads.onCreated.addListener(async (downloadItem) => {
    await browser.downloads.cancel(downloadItem.id);
    const message = {
      url: downloadItem.url,
      filename: downloadItem.filename,
      mime: downloadItem.mime,
      referrer: downloadItem.referrer,
      headers: downloadItem.headers,
    };
    port.postMessage(message);
  });

  port.onMessage.addListener((response) => {
    console.log("Received from native app:", response);
  });
}
