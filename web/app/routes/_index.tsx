import type { MetaFunction } from "@remix-run/node";
import { api } from "~/lib/api";
import { useEffect } from "react";

export const meta: MetaFunction = () => {
  return [
    { title: "New Remix App" },
    { name: "description", content: "Welcome to Remix!" },
  ];
};

export default function Index() {
  useEffect(() => {
    let unsubscribe = api.addSubscription(["pings"], {
      onData: (data) => {
        console.log("Ping:", data);
      },
      onError: (error) => {
        console.error("Ping error:", error);
      },
    });
    return () => {
      unsubscribe();
    };
  }, []);
  return (
    <div style={{ fontFamily: "system-ui, sans-serif", lineHeight: "1.8" }}>
      <h1>Welcome to Remix</h1>
      <button
        onClick={async () => {
          let promises = [];
          console.time("500 queries");
          for (let i = 0; i < 500; i++) {
            promises.push(api.query(["version"]));
          }
          await Promise.all(promises);
          console.timeEnd("500 queries");
        }}
      >
        Test
      </button>
      <ul>
        <li>
          <a
            target="_blank"
            href="https://remix.run/tutorials/blog"
            rel="noreferrer"
          >
            15m Quickstart Blog Tutorial
          </a>
        </li>
        <li>
          <a
            target="_blank"
            href="https://remix.run/tutorials/jokes"
            rel="noreferrer"
          >
            Deep Dive Jokes App Tutorial
          </a>
        </li>
        <li>
          <a target="_blank" href="https://remix.run/docs" rel="noreferrer">
            Remix Docs
          </a>
        </li>
      </ul>
    </div>
  );
}
