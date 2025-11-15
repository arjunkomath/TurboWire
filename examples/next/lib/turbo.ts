import { createTurboWireHub } from "@turbowire/serverless";
import { chatSchema } from "./schema";

if (!process.env.TURBOWIRE_DOMAIN) {
  throw new Error("TURBOWIRE_DOMAIN environment variable is not set");
}

export const turboWireHub = createTurboWireHub(process.env.TURBOWIRE_DOMAIN, {
  schema: chatSchema,
  secure: false, // for local testing
});
