[CmdletBinding(DefaultParameterSetName = "Path", HelpURI = "https://go.microsoft.com/fwlink/?LinkId=517145")]
param(
	[Parameter(ParameterSetName="Path", Position = 0)]
	[System.String[]]
	$Path


)

begin
{
	# Construct the strongly-typed crypto object
}

process
{
	Write-output elo
}
	