# Privacy Policy and User Rights

## Your Privacy Rights

TGraph Bot respects your privacy and is committed to protecting your personal data. This document explains what data we collect, how we use it, and your rights regarding your data.

## Data We Collect

### User Preferences
- Discord user ID (for identification)
- Privacy settings (username visibility, data retention preferences)
- Language preferences
- Direct message delivery preferences
- Timestamps of when preferences were created and last updated

### Command Usage Statistics
- Command execution history (command name, timestamp, success/failure)
- Response times and performance metrics
- Channel and server usage patterns
- Activity patterns (hourly and daily usage)

### Audit Logs
- Records of data access, modification, export, and deletion requests
- GDPR compliance activities
- Privacy setting changes

## How We Use Your Data

- **Statistics Generation**: To provide you with personal usage statistics via the `/my_stats` command
- **Performance Monitoring**: To improve bot performance and reliability
- **Privacy Compliance**: To respect your privacy preferences and data retention policies
- **Audit Trail**: To maintain records of data protection activities for compliance

## Your Rights Under GDPR

### Right to Access
You can request a copy of all data we have stored about you using the `/export_my_data` command. This will provide you with a complete export in JSON format containing:
- Your user preferences and privacy settings
- Complete command execution history
- Aggregated statistics
- Audit log entries related to your account

### Right to Deletion
You can request permanent deletion of all your data using the `/delete_my_data` command. This will:
- Permanently remove your user preferences and privacy settings
- Clear your command execution history
- Remove cached statistics and activity data
- Clear audit log entries related to your account
- Remove any other stored personal information

**⚠️ Warning**: Data deletion is permanent and cannot be undone.

### Right to Rectification
Contact a server administrator if you need to correct any inaccurate personal data.

### Right to Data Portability
The `/export_my_data` command provides your data in a portable JSON format that can be used with other services.

### Right to Restrict Processing
You can adjust your privacy settings to restrict how your data is processed:
- Disable public statistics sharing
- Set data retention periods
- Control direct message delivery preferences

## Data Retention

- **Default Policy**: Data is retained indefinitely unless you specify otherwise
- **Custom Retention**: You can set a custom data retention period in your preferences
- **Automatic Cleanup**: Data is automatically deleted when it exceeds your specified retention period
- **Manual Deletion**: You can delete your data at any time using `/delete_my_data`

## Privacy Settings

### Username Visibility
- Control whether your username appears in statistics
- Default: Visible

### Public Statistics
- Control whether your statistics can be shared publicly
- Default: Private (not shared publicly)

### Direct Message Delivery
- Choose whether to receive statistics via direct message for enhanced privacy
- Default: Enabled (statistics sent via DM)

### Data Export
- Control whether you can export your data
- Default: Enabled

## How to Exercise Your Rights

### Export Your Data
```
/export_my_data
```
- Generates a complete export of all your data
- Sent via direct message for privacy
- Includes all preferences, statistics, and audit logs
- 5-minute cooldown between requests

### Delete Your Data
```
/delete_my_data confirmation:CONFIRM
```
- Permanently deletes ALL your data
- Requires explicit confirmation
- Cannot be undone
- 10-minute cooldown between requests
- Detailed confirmation sent via direct message

### View Your Statistics
```
/my_stats [period]
```
- View your personal usage statistics
- Available periods: daily, weekly, monthly, all-time
- Respects your privacy settings
- Can be delivered via DM based on your preferences

## Contact Information

If you have questions about your privacy rights or need assistance with data requests:
- Contact a server administrator
- Use the bot's help commands for technical support
- Refer to this documentation for self-service options

## Data Security

- All data is stored securely using encrypted databases
- Access is logged and audited for compliance
- Regular security reviews and updates
- Minimal data collection principle (only what's necessary)

## Changes to This Policy

This privacy policy may be updated from time to time. Users will be notified of significant changes through appropriate channels.

## Compliance

This bot is designed to comply with:
- General Data Protection Regulation (GDPR)
- Discord Terms of Service
- Data protection best practices

---

*Last updated: December 2024*
*For technical questions about privacy features, contact the bot administrators.* 