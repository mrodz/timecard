import { use, memo, useState } from "react";
import { ClockSchema } from "@/lib/api";
import UserClock from "./UserClock";
import CreateClock from "./modals/CreateClock";

type ClockListProps = {
	loadAllClocks: Promise<ClockSchema[]>
}

const ClockList = memo((props: ClockListProps) => {
	const initialClocks = use(props.loadAllClocks)

	const [clocks, setClocks] = useState(initialClocks)

	const [creatingClock, setCreatingClock] = useState<string | null>(null);

	const onClockCreationStart = (name: string) => setCreatingClock(name);
	const onClockCreated = (clock: ClockSchema) => {
		setCreatingClock(null)
		setClocks((list) => {
			list.push(clock);
			return list;
		})
	}

	return (
		<div className="flex flex-col gap-4">
			<div>
				<CreateClock onClockCreationStart={onClockCreationStart} onClockCreated={onClockCreated} />
			</div>
			<div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
				{creatingClock && <UserClock skeleton={creatingClock} />}

				{clocks.sort((a, b) => b.last_edit.valueOf() - a.last_edit.valueOf()).map((clock, i) => <UserClock clock={clock} key={i} />)}
			</div>
		</div>
	)
});

export default ClockList;