use anchor_lang::prelude::*;

#[error_code]
pub enum PlatformError {
    #[msg("Platform is currently inactive")]
    PlatformInactive, 

     #[msg("Arithmetic overflow")]
    ArithmeticOverflow,

    #[msg("Invalid fee percentage")]
    InvalidFeePercentage,
      
    #[msg("Platform is already initialized")]
    PlatformAlreadyInitialized, 
    
    #[msg("Campaign is not active")]
    CampaignNotActive,
    
    #[msg("Campaign is still active")]
    CampaignStillActive,
    
    #[msg("Campaign has not started yet")]
    CampaignNotStarted,

    #[msg("Campaign cannot be closed yet")]
    CampaignCannotBeClosed,
    
    #[msg("Campaign has already ended")]
    CampaignEnded,
    
    #[msg("Invalid campaign duration. End time must be after start time")]
    InvalidCampaignDuration,
    
    #[msg("Insufficient deposit amount. Minimum required")]
    InsufficientDepositAmount,
    
    #[msg("Campaign already exists for this merchant and product")]
    CampaignAlreadyExists,
    
    #[msg("Campaign not found")]
    CampaignNotFound,

    #[msg("Only platform admin can perform this action")]
    UnauthorizedAdmin,

    #[msg("Only campaign merchant can perform this action")]
    UnauthorizedMerchant,

    #[msg("Only campaign reviewer can perform this action")]
    UnauthorizedReviewer,

    #[msg("Reviewer cannot review their own campaign")]
    CannotReviewOwnCampaign,
    
    #[msg("User has not participated in this campaign")]
    UserNotParticipated,
    
    #[msg("User has already participated in this campaign")]
    AlreadyParticipated, 
    
    #[msg("Insufficient user reputation to participate")]
    InsufficientReputation,

    #[msg("Review already exists for this user and campaign")]
    ReviewAlreadyExists,
    
    #[msg("Review not found")]
    ReviewNotFound, 
    
    #[msg("Review description too long. Maximum 500 characters")]
    ReviewDescriptionTooLong, 

    #[msg("Review description too long. Maximum 64 characters")]
    ReasonTooLong, 
    
    #[msg("Review description cannot be empty")]
    EmptyReviewDescription, 
    
    #[msg("Review has already been approved")]
    ReviewAlreadyApproved, 
    
    #[msg("Review has been flagged and cannot be approved")]
    ReviewFlagged, 
    
    #[msg("Only pending reviews can be approved")]
    CannotApproveReview, 
    
    #[msg("Invalid transaction ID format")]
    InvalidTransactionId, 
    
    #[msg("Review is not approved for reward claim")]
    ReviewNotApproved, 

    #[msg("Refund is not availabe, reviews needed reached")]
    NoRefundAvailable, 

    #[msg("Merchant's reviews needed reached")]
    CampaignTargetMet, 
    
    #[msg("Reward already claimed for this review")]
    RewardAlreadyClaimed, 

    #[msg("Insufficient funds in campaign vault")]
    InsufficientVaultFunds, 
    
    #[msg("Insufficient funds in treasury")]
    InsufficientTreasuryFunds, 
    
    #[msg("REV tokens are non-transferable")]
    TokensNonTransferable, 
    
    #[msg("Reward calculation error")]
    RewardCalculationError, 
    
    #[msg("Fee calculation error")]
    FeeCalculationError, 
}