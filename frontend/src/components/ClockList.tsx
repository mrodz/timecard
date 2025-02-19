import { use, memo } from "react";
import { type ClockSchema } from "@/lib/api";
import UserClock from "./UserClock";

type ClockListProps = {
	loadAllClocks: Promise<ClockSchema[]>
}

const ClockList = memo((props: ClockListProps) => {
	const clocks = use(props.loadAllClocks)

	return (
		<div>
			{clocks.map((clock, i) => <UserClock clock={clock} key={i} />)}
		</div>
	)
});

export default ClockList;