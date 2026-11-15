// The Microsoft Store package (MSIX) of Windows, made from the release build that
// `npm run package` left (decided by Carlos on 2026-11-15: sell through the Store,
// which signs the package). Usage, after `npm run package` in app/:
//
//   node scripts/make-msix.mjs
//
// The package's identity comes from Partner Center once the account exists; until then
// test values. Set them with XS_MSIX_NAME, XS_MSIX_PUBLISHER (CN=…) and
// XS_MSIX_PUBLISHER_NAME. The package is not signed: the Store signs it when it is
// published. To try it on this computer: Windows' Developer Mode, then
// `Add-AppxPackage -Register target\release\msix\XcreenShot\AppxManifest.xml`.
import { execFileSync } from "node:child_process";
import {
  copyFileSync,
  existsSync,
  mkdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

if (process.platform !== "win32") {
  console.log("The MSIX package is only made on Windows.");
  process.exit(0);
}
const root = fileURLToPath(new URL("..", import.meta.url));
const app = join(root, "app");
const version = JSON.parse(
  readFileSync(join(app, "package.json"), "utf8"),
).version;
const release = join(root, "target", "release");
const out = join(release, "msix");
const stage = join(out, "XcreenShot");
rmSync(out, { recursive: true, force: true });
mkdirSync(join(stage, "Assets"), { recursive: true });

const name = process.env.XS_MSIX_NAME ?? "XcreenShot.Test";
const publisher = process.env.XS_MSIX_PUBLISHER ?? "CN=XcreenShot Test";
const publisherName = process.env.XS_MSIX_PUBLISHER_NAME ?? "XcreenShot";

// The app, the CLI (with its own alias, APP-028) and the guide for agents.
copyFileSync(
  join(release, "xcreenshot-app.exe"),
  join(stage, "xcreenshot-app.exe"),
);
copyFileSync(join(release, "xcreenshot.exe"), join(stage, "xcreenshot.exe"));
copyFileSync(join(root, "AGENTS.md"), join(stage, "AGENTS.md"));

// The tiles and the store logo, drawn from the app's icon.
const icons = join(out, "icons");
execFileSync(
  "npx",
  ["tauri", "icon", join("src-tauri", "icons", "app-icon.png"), "-o", icons],
  {
    cwd: app,
    stdio: "ignore",
    shell: true,
  },
);
for (const f of [
  "Square44x44Logo.png",
  "Square150x150Logo.png",
  "Square310x310Logo.png",
  "StoreLogo.png",
]) {
  if (existsSync(join(icons, f)))
    copyFileSync(join(icons, f), join(stage, "Assets", f));
}

const types = [
  ["xshot", "XcreenShot capture"],
  ["xpreset", "XcreenShot presets"],
  ["xtheme", "XcreenShot theme"],
  ["xtemplate", "XcreenShot template"],
  ["xsettings", "XcreenShot settings"],
  ["xstamps", "XcreenShot stamp pack"],
];
const associations = types
  .map(
    ([
      ext,
      label,
    ]) => `        <uap:Extension Category="windows.fileTypeAssociation">
          <uap:FileTypeAssociation Name="${ext}">
            <uap:DisplayName>${label}</uap:DisplayName>
            <uap:SupportedFileTypes><uap:FileType>.${ext}</uap:FileType></uap:SupportedFileTypes>
          </uap:FileTypeAssociation>
        </uap:Extension>`,
  )
  .join("\n");

const manifest = `<?xml version="1.0" encoding="utf-8"?>
<Package
  xmlns="http://schemas.microsoft.com/appx/manifest/foundation/windows10"
  xmlns:uap="http://schemas.microsoft.com/appx/manifest/uap/windows10"
  xmlns:uap5="http://schemas.microsoft.com/appx/manifest/uap/windows10/5"
  xmlns:desktop="http://schemas.microsoft.com/appx/manifest/desktop/windows10"
  xmlns:rescap="http://schemas.microsoft.com/appx/manifest/foundation/windows10/restrictedcapabilities"
  IgnorableNamespaces="uap uap5 desktop rescap">
  <Identity Name="${name}" Publisher="${publisher}" Version="${version}.0" ProcessorArchitecture="x64" />
  <Properties>
    <DisplayName>XcreenShot</DisplayName>
    <PublisherDisplayName>${publisherName}</PublisherDisplayName>
    <Logo>Assets\\StoreLogo.png</Logo>
  </Properties>
  <Dependencies>
    <TargetDeviceFamily Name="Windows.Desktop" MinVersion="10.0.19045.0" MaxVersionTested="10.0.26100.0" />
  </Dependencies>
  <Resources>
    <Resource Language="en-us" />
    <Resource Language="es-es" />
  </Resources>
  <Applications>
    <Application Id="XcreenShot" Executable="xcreenshot-app.exe" EntryPoint="Windows.FullTrustApplication">
      <uap:VisualElements DisplayName="XcreenShot" Description="Capture, annotate and record your screen"
        BackgroundColor="transparent" Square150x150Logo="Assets\\Square150x150Logo.png" Square44x44Logo="Assets\\Square44x44Logo.png" />
      <Extensions>
        <desktop:Extension Category="windows.startupTask" Executable="xcreenshot-app.exe" EntryPoint="Windows.FullTrustApplication">
          <desktop:StartupTask TaskId="XcreenShotStartup" Enabled="false" DisplayName="XcreenShot" />
        </desktop:Extension>
${associations}
      </Extensions>
    </Application>
    <Application Id="XcreenShotCli" Executable="xcreenshot.exe" EntryPoint="Windows.FullTrustApplication">
      <uap:VisualElements DisplayName="XcreenShot CLI" Description="The xcreenshot command" AppListEntry="none"
        BackgroundColor="transparent" Square150x150Logo="Assets\\Square150x150Logo.png" Square44x44Logo="Assets\\Square44x44Logo.png" />
      <Extensions>
        <uap5:Extension Category="windows.appExecutionAlias" Executable="xcreenshot.exe" EntryPoint="Windows.FullTrustApplication">
          <uap5:AppExecutionAlias><uap5:ExecutionAlias Alias="xcreenshot.exe" /></uap5:AppExecutionAlias>
        </uap5:Extension>
      </Extensions>
    </Application>
  </Applications>
  <Capabilities>
    <rescap:Capability Name="runFullTrust" />
    <DeviceCapability Name="microphone" />
    <DeviceCapability Name="webcam" />
  </Capabilities>
</Package>
`;
writeFileSync(join(stage, "AppxManifest.xml"), manifest);

const kits = "C:\\Program Files (x86)\\Windows Kits\\10\\bin";
const sdk = readdirVersions(kits).find((v) =>
  existsSync(join(kits, v, "x64", "makeappx.exe")),
);
if (!sdk) {
  console.error("makeappx.exe not found: install the Windows SDK.");
  process.exit(1);
}
const msix = join(out, `XcreenShot_${version}_x64.msix`);
execFileSync(
  join(kits, sdk, "x64", "makeappx.exe"),
  ["pack", "/o", "/d", stage, "/p", msix],
  {
    stdio: "inherit",
  },
);
console.log(`MSIX: ${msix}`);

function readdirVersions(dir) {
  try {
    return execFileSync("cmd", ["/c", "dir", "/b", "/ad", dir], {
      encoding: "utf8",
    })
      .split(/\r?\n/)
      .filter((v) => /^10\.0\.\d+\.\d+$/.test(v))
      .sort()
      .reverse();
  } catch {
    return [];
  }
}
