# TurboWire Next.js Demo

This is a real-time chat application demonstrating TurboWire's capabilities.

## Setup

1. **Configure environment variables:**

   ```bash
   cp .env.local.example .env.local
   ```

   Edit `.env.local` with your TurboWire server credentials:

   ```env
   TURBOWIRE_DOMAIN=localhost:8080
   TURBOWIRE_BROADCAST_KEY=your-broadcast-key
   TURBOWIRE_SIGNING_KEY=your-signing-key
   ```

2. **Install dependencies:**

   ```bash
   pnpm install
   ```

3. **Start the TurboWire server** (in the repository root):

   ```bash
   cd ../../apps/server
   cargo run
   ```

4. **Start the Next.js app:**

   ```bash
   pnpm dev
   ```

5. **Open the app:**

   Navigate to [http://localhost:3000](http://localhost:3000)
