export const config = { 
    serverUrl: "http://127.0.0.1:8000/",
    notesSince: 7,
    // Either make it or change to Reminders. We can't tell if the list doesn't exist, in which case the script crashes :/ 
    remindersList: "Notes",
    userId: "avikam@gmail.com",
    includeFolders: [
        "Weekly goals"
    ]
}
