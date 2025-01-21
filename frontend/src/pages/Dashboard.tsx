import {
	CognitoUserPool,
	CognitoUserAttribute,
	CognitoUser,
	CognitoUserSession,
} from 'amazon-cognito-identity-js';
import { Component } from 'react';
import { Outlet } from 'react-router-dom';
import useAuth from "@/lib/useAuth"
import useAttributes from '@/lib/useAttributes';

export default function Dashboard() {
	const { userPool } = useAuth()

	const user = userPool.getCurrentUser();

	// const [userAttributes] = useAttributes(user!);

	return (
		<div>
			{JSON.stringify(user)}

			<Outlet />
		</div>
	)
}