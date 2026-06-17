# Credit Methodology

## Carbon Credit Calculation

credits_per_period = (carbon_sequestered_kg / 1000) * credit_conversion_factor

Where `credit_conversion_factor` is set at bond issuance per methodology:
- VERRA-VCS: 1.0 (standard)
- GOLD-STANDARD: 1.0
- ACR: 0.95 (conservative)
- CAR: 1.05 (includes buffer pool)

## Biodiversity Credit Calculation

Biodiversity credits are calculated using project-specific metrics:
- Habitat hectares restored
- Species Abundance Index (SAI) improvement
- Biodiversity Unit (UK BNG methodology)

## Oracle Data Sources
- Accredited Auditors: annual baseline verification
- Satellite Imagery: monthly NDVI/biomass proxy
- IoT Sensors: continuous soil carbon/moisture
- Community Monitors: quarterly species surveys
