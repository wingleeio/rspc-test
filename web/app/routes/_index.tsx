import type { MetaFunction } from "@remix-run/node";
import { useEffect } from "react";
import { api } from "~/lib/api";

export const meta: MetaFunction = () => {
  return [{ title: "New Remix App" }, { name: "description", content: "Welcome to Remix!" }];
};

export default function Index() {
  useEffect(() => {
    api.query(["version"]).then((result) => {
      console.log("Server version:", result);
    });

    let test = api.addSubscription(["pings"], {
      onData: (data) => {
        console.log("Ping:", data);
      },
      onError: (error) => {
        console.error("Ping error:", error);
      },
    });
    return () => {
      test();
    };
  }, []);
  return (
    <div style={{ fontFamily: "system-ui, sans-serif", lineHeight: "1.8" }}>
      <h1>Welcome to Remix</h1>
      <ul>
        <li>
          <a target="_blank" href="https://remix.run/tutorials/blog" rel="noreferrer">
            15m Quickstart Blog Tutorial
          </a>
        </li>
        <li>
          <a target="_blank" href="https://remix.run/tutorials/jokes" rel="noreferrer">
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
