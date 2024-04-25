import { httpLink, initRspc, wsLink } from "@rspc/client";
import { Procedures } from "~/generated/bindings";

const DOMAIN = "localtest.me";
const WS_URL = `ws://${DOMAIN}:4000/ws`;
const RPC_URL = `http://${DOMAIN}:4000`;

export const api = initRspc<Procedures>({
  links: [
    ({ op, next }) => {
      if (op.method === "subscription") {
        return wsLink({ url: WS_URL })({ op, next });
      }

      return next({ op });
    },
    httpLink({
      url: RPC_URL,
      fetch: (input, init) => {
        return fetch(input, {
          ...init,
          credentials: "include",
        });
      },
    }),
  ],
  onError: (error) => {
    console.error(error);
  },
});
