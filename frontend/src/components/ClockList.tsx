import { use, useState } from "react";
import { ClockSchema } from "@/lib/api";
import UserClock from "./UserClock";
import CreateClock from "./modals/CreateClock";

type ClockListProps = {
	loadAllClocks: Promise<ClockSchema[]>
}

const ClockList = (props: ClockListProps) => {
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

	const onClockEdit = (newClock: ClockSchema) => {
		setClocks((clocks) => {
			for (let i = 0; i < clocks.length; i++) {
				if (clocks[i].uuid === newClock.uuid) {
					clocks[i] = newClock
					return [...clocks]
				}
			}
			// never
			return clocks
		})
	}

	const onClockDeleteStart = (deletedIndex: number) => {
		setClocks((clocks) => {
			delete clocks[deletedIndex]
			return [...clocks]
		})
	}

	return (
		<div className="flex flex-col gap-4">
			<div>
				<CreateClock onClockCreationStart={onClockCreationStart} onClockCreated={onClockCreated} />
			</div>
			<div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
				{creatingClock && <UserClock skeleton={creatingClock} />}

				{clocks.sort((a, b) => b.last_edit.valueOf() - a.last_edit.valueOf()).map((clock, i) => {
					if (!clock) return null // the `delete` runs before the state change is propagated, so ignore ghost entry

					return (
						<UserClock onDeleteStart={() => onClockDeleteStart(i)} onEdit={onClockEdit} clock={clock} key={clock.uuid} />
					)
				})}
			</div>
		</div>
	)
};

export default ClockList;