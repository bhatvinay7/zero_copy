const redis = require("redis");

const client = redis.createClient({
  url: "rediss://default:AVNS_UVwBys4eDWSjaUHrAiW@valkey-150db72f-bookit.a.aivencloud.com:26721"
});

client.on("error", (err) => console.log("Redis Client Error", err));

async function run() {
  console.log("Connecting...");
  await client.connect();
  console.log("Connected!");
  await client.set("test-key", "test-value");
  const value = await client.get("test-key");
  console.log("Got value:", value);
  await client.disconnect();
}

run();
