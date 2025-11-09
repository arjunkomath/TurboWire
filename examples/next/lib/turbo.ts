import { TurboWireHub } from "@turbowire/serverless";
import { chatSchema } from "./schema";

export const turboWireHub = new TurboWireHub(process.env.TURBOWIRE_DOMAIN!, {
  schema: chatSchema,
  secure: false, // for local testing
});
