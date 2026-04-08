import { memo } from "react";
import { CircleAlert, CircleCheckBig, Info, X } from "lucide-react";

export interface ToastItem {
  id: number;
  tone: "error" | "success" | "info";
  message: string;
}

function ToastStackComponent({
  items,
  onDismiss,
}: {
  items: ToastItem[];
  onDismiss: (id: number) => void;
}) {
  if (items.length === 0) {
    return null;
  }

  return (
    <div className="toast toast-top toast-end z-50">
      {items.map((item) => (
        <div
          key={item.id}
          className={`alert max-w-sm shadow-lg ${
            item.tone === "error"
              ? "alert-error"
              : item.tone === "success"
                ? "alert-success"
                : "alert-info"
          }`}
        >
          {item.tone === "error" ? (
            <CircleAlert size={18} className="shrink-0" />
          ) : item.tone === "success" ? (
            <CircleCheckBig size={18} className="shrink-0" />
          ) : (
            <Info size={18} className="shrink-0" />
          )}
          <span className="text-sm leading-5">{item.message}</span>
          <button
            type="button"
            className="btn btn-ghost btn-xs btn-circle text-current"
            aria-label="Dismiss notification"
            onClick={() => onDismiss(item.id)}
          >
            <X size={14} />
          </button>
        </div>
      ))}
    </div>
  );
}

export const ToastStack = memo(ToastStackComponent);
