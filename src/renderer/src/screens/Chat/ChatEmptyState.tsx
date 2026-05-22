import { memo } from "react";
import { Search, Clock, Mail, Code, ChartLine, Bell } from "lucide-react";
import icon from "../../assets/icon.png";
import { useI18n } from "../../components/useI18n";

interface Suggestion {
  i18nKey: string;
  Icon: typeof Search;
}

const SUGGESTIONS: Suggestion[] = [
  {
    i18nKey: "chat.suggestionSearch",
    Icon: Search,
  },
  {
    i18nKey: "chat.suggestionReminder",
    Icon: Bell,
  },
  {
    i18nKey: "chat.suggestionEmail",
    Icon: Mail,
  },
  {
    i18nKey: "chat.suggestionScript",
    Icon: Code,
  },
  {
    i18nKey: "chat.suggestionSchedule",
    Icon: Clock,
  },
  {
    i18nKey: "chat.suggestionAnalyze",
    Icon: ChartLine,
  },
];

interface ChatEmptyStateProps {
  onSelectSuggestion: (text: string) => void;
}

export const ChatEmptyState = memo(function ChatEmptyState({
  onSelectSuggestion,
}: ChatEmptyStateProps): React.JSX.Element {
  const { t } = useI18n();

  return (
    <div className="chat-empty">
      <div className="chat-empty-icon">
        <img src={icon} width={64} height={64} alt="" />
      </div>
      <div className="chat-empty-text">{t("chat.emptyTitle")}</div>
      <div className="chat-empty-hint">{t("chat.emptyHint")}</div>
      <div className="chat-empty-suggestions">
        {SUGGESTIONS.map(({ i18nKey, Icon }) => (
          <button
            key={i18nKey}
            className="chat-suggestion"
            onClick={() => onSelectSuggestion(t(i18nKey))}
          >
            <Icon size={16} />
            {t(i18nKey)}
          </button>
        ))}
      </div>
    </div>
  );
});
