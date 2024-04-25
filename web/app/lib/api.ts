import { httpLink, initRspc, wsLink } from "@rspc/client";
import { Procedures } from "~/generated/bindings";

export const api = initRspc<Procedures>({
  links: [
    ...(typeof window !== "undefined"
      ? [
          wsLink({
            url: "ws://localhost:4000/ws",
          }),
        ]
      : []),
    httpLink({
      url: "http://localhost:4000",
      headers: {
        "Content-Type": "application/json",
      },
    }),
  ],
  onError: (error) => {
    console.error("API error:", error);
  },
});
