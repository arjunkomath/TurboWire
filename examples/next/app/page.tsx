import ChatRoom from "@/components/ChatRoom";
import { ROOM } from "@/lib/schema";
import { turboWireHub } from "@/lib/turbo";
import randomName from "@scaleway/random-name";

export default function Home() {
  const userId = randomName();
  const wireUrl = turboWireHub.getSignedWire(ROOM);

  return (
    <div className="flex-row items-center justify-center">
      <ChatRoom userId={userId} wireUrl={wireUrl} />
    </div>
  );
}
