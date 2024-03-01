# APOD at the Edge

This is a simple Fastly Compute@Edge service to get the information pertaining to the most recent
`Astronomy Picture of the Day Image` between two dates.

It was written for the purposes of exploring Fastly's Edge Compute ecosystem.

## Usage 

```
curl -v -H "Fastly-Key:<>" -H "x-custom-apod-api-key:<>" <service-domain>
```

