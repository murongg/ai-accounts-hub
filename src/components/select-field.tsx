export interface SelectFieldOption {
  label: string;
  value: string;
}

export function SelectField({
  value,
  onChange,
  options,
  isDisabled = false,
  className,
}: {
  value: string;
  onChange: (value: string) => void;
  options: SelectFieldOption[];
  isDisabled?: boolean;
  className?: string;
}) {
  return (
    <select
      value={value}
      disabled={isDisabled}
      onChange={(event) => onChange(event.target.value)}
      className={`select select-bordered w-full rounded-xl border-base-300 bg-base-100 text-sm shadow-none focus:outline-none ${className ?? ""}`}
    >
      {options.map((option) => (
        <option key={option.value} value={option.value}>
          {option.label}
        </option>
      ))}
    </select>
  );
}
