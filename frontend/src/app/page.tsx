import rawHtml from "./exact_harness.html?raw";
import { HarnessInteractiveBridge } from "@/components/HarnessInteractiveBridge";

export default function Home() {
  return (
    <>
      <div
        className="min-h-screen bg-[#0a0a0a]"
        dangerouslySetInnerHTML={{ __html: rawHtml }}
        suppressHydrationWarning
      />
      <HarnessInteractiveBridge />
    </>
  );
}
