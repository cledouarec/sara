---
id: "SCEN-001"
type: scenario
name: "Successful Login"
description: "User logs in with valid credentials"
refines:
  - "UC-001"
---

# Scenario: Successful Login

## Overview

User enters valid credentials and gains access to the portal.

## Initial State

- User is on login page
- User has valid credentials

## Trigger

User clicks the "Login" button after entering credentials.

## Step-by-Step Flow

1. User enters username
2. User enters password
3. User clicks "Login" button
4. System validates credentials
5. System creates session
6. System redirects to dashboard

## Expected Outcome

- **Success Condition**: User sees dashboard
- **Verification**: Session token is valid

## Exceptions & Edge Cases

- Invalid credentials: Show error message
- Account locked: Show lockout message
