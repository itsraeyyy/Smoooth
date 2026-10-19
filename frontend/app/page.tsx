import fs from "fs";
import path from "path";
import { HarnessInteractiveBridge } from "../components/HarnessInteractiveBridge";

export default function Home() {
  const candidates = [
    path.join(process.cwd(), "app/exact_harness.html"),
    path.join(process.cwd(), "frontend/app/exact_harness.html"),
    "C:/Users/HP/Downloads/Smoooth/frontend/app/exact_harness.html",
    "C:/Users/HP/Downloads/Smoooth/scratch/exact_full_page.html"
  ];

  let rawHtml = "";
  for (const p of candidates) {
    if (fs.existsSync(p)) {
      rawHtml = fs.readFileSync(p, "utf-8");
      break;
    }
  }

  return (
    <main className="min-h-screen bg-[#080C14]">
      <div dangerouslySetInnerHTML={{ __html: rawHtml }} />
      <HarnessInteractiveBridge />
    </main>
  );
}
