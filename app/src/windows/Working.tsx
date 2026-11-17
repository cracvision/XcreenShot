// "Working" next to the zone while Extract text reads it (SMT-005).
import { useEffect } from "react";
import { useTranslation } from "react-i18next";

export function Working() {
  const { t } = useTranslation();
  useEffect(() => {
    document.documentElement.classList.add("launcher-root");
  }, []);
  return (
    <div className="working-pill" role="status">
      <span className="working-dot" aria-hidden="true" />
      {t("ui.extract.working")}
    </div>
  );
}
