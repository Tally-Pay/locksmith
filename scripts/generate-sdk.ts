import { createFromRoot } from "codama";
import { rootNodeFromAnchor } from "@codama/nodes-from-anchor";
import { renderVisitor as renderJavaScriptVisitor } from "@codama/renderers-js";
import path from "path";
import { fileURLToPath } from "url";

// ESM equivalent of __dirname
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Load the Shank IDL
const idlPath = path.join(__dirname, "..", "idl", "locksmith.json");
const idl = await import(idlPath, { with: { type: "json" } });

console.log("Loaded IDL from:", idlPath);
console.log("Program:", idl.default.name);
console.log("Instructions:", idl.default.instructions.length);
console.log("Accounts:", idl.default.accounts.length);

// Create Codama tree from the IDL
// Note: @codama/nodes-from-anchor works with Shank IDLs when origin is "shank"
const codama = createFromRoot(rootNodeFromAnchor(idl.default));

// Output directory for generated SDK
const sdkOutputDir = path.join(__dirname, "..", "sdk", "src", "generated");

console.log("\nGenerating TypeScript SDK to:", sdkOutputDir);

// Generate the JavaScript/TypeScript client
codama.accept(renderJavaScriptVisitor(sdkOutputDir));

console.log("\nSDK generation complete!");
