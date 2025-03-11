import { useState } from "react";

export default function useDateFormat(locale: Intl.LocalesArgument = "en-US") {
	const [reactiveLocale, setFormatterLocale] = useState(locale);

	return {
		formatter: {
			date: new Intl.DateTimeFormat(reactiveLocale, {
				year: "numeric",
				month: "short",
				day: "numeric",
				hour: "numeric",
				minute: "numeric",
				hour12: true,
			}),
			minute: new Intl.DateTimeFormat(reactiveLocale, {
				year: "numeric",
				month: "short",
				day: "numeric",
				hour: "numeric",
				minute: "numeric",
				second: "numeric",
				hour12: true,
			}),
		},
		setFormatterLocale
	}
}