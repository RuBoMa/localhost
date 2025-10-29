export function formatDisplayDate(dateString) {
    const [year, month, day] = dateString.split('T')[0].split('-');
    return `${day}-${month}-${year}`;
}

export function getLastPathSegment(fullPath) {
    const parts = fullPath.split('/');
    return parts[parts.length - 1] || fullPath;
}

export function formatMonthYear(dateString) {
    const date = new Date(dateString);
    const mm = String(date.getMonth() + 1).padStart(2, '0');
    const yy = String(date.getFullYear()).slice(-2);
    return `${mm}/${yy}`;
}