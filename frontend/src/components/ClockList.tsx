import { use, memo, useState } from "react";
import { createUserClock, type ClockSchema } from "@/lib/api";
import UserClock from "./UserClock";
import { Button } from "@/components/ui/button";
import { CurrentUserContext } from "@/pages/Layout";

type ClockListProps = {
	loadAllClocks: Promise<ClockSchema[]>
}

const ClockList = memo((props: ClockListProps) => {
	const user = use(CurrentUserContext)!
	const initialClocks = use(props.loadAllClocks)

	const [clocks, setClocks] = useState(initialClocks)

	const [creatingClock, setCreatingClock] = useState<string | null>(null);

	const createClock = async () => {
		const name = window.prompt("Enter name");

		if (!name) {
			window.alert("cannot be empty")
			return;
		}

		setCreatingClock(name)

		const clock = await createUserClock({
			userPoolId: user.reactiveUser!.getUsername()!,
			name,
		})

		setCreatingClock(null)

		setClocks((list) => {
			list.push(clock);
			return list;
		})
	}

	return (
		<div className="flex flex-row gap-4">
			{clocks.map((clock, i) => <UserClock clock={clock} key={i} />)}

			{creatingClock && <UserClock skeleton name={creatingClock} />}

			<Button className="self-center" onClick={createClock}>Create Clock</Button>
		</div>
	)
});

export default ClockList;