import { useEffect, useState } from "react";
import { hermesAPI } from "@shared/hermes-api";

function Versions(): React.JSX.Element {
  const [info, setInfo] = useState<{
    appVersion: string;
    platform: string;
    arch: string;
  } | null>(null);

  useEffect(() => {
    hermesAPI
      .getSystemInfo()
      .then((sys) =>
        setInfo({
          appVersion: sys.appVersion,
          platform: sys.platform,
          arch: sys.arch,
        }),
      )
      .catch(() => {});
  }, []);

  return (
    <ul className="versions">
      <li className="electron-version">
        Hermes Desktop v{info?.appVersion || "—"}
      </li>
      <li className="chrome-version">
        {info?.platform || "—"} / {info?.arch || "—"}
      </li>
    </ul>
  );
}

export default Versions;
