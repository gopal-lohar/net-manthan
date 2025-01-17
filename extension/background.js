// Open a connection to the native host
const port = browser.runtime.connectNative("com.down_poc.native_messaging");

// Send a test message
port.postMessage({ message: "Hello from down-poc extension!" });

// Listen for messages from the native host
port.onMessage.addListener((response) => {
  console.log("Received from native host:", response);
});

// Handle disconnection
port.onDisconnect.addListener(() => {
  console.error("Disconnected from native host");
});

