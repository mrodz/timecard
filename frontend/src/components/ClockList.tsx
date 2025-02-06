import { useEffect, use, memo, useMemo, useCallback } from "react";
import { Card, CardContent, CardDescription, CardTitle } from "./ui/card";
import { loadAllUserClocks, type ClockSchema } from "@/lib/api";
import { CurrentUserContext } from "@/pages/Layout";

type ClockListProps = {
	loadAllClocks: Promise<ClockSchema[]>
}

const ClockList = memo((props: ClockListProps) => {
	const clocks = use(props.loadAllClocks)

	return (
		<div>
			{clocks.map((clock, i) => (
				<Card key={i} className="w-1/6 p-4">
					<CardTitle>{clock.name}</CardTitle>
					<CardDescription>{clock.uuid}</CardDescription>

					<CardContent>
						<pre className="text-wrap overflow-x-scroll">
							{JSON.stringify(clock)}
						</pre>
					</CardContent>
				</Card>
			))}
		</div>
	)
});

export default ClockList;