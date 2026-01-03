// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import { Initializable } from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import { UUPSUpgradeable } from "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import { ReentrancyGuardUpgradeable } from "@openzeppelin/contracts-upgradeable/utils/ReentrancyGuardUpgradeable.sol";

import { IStrovaCreditToken } from "./IStrovaCreditToken.sol";

/// @notice EVM/Base port of the Solana `sync_contract` program.
/// Keyed storage is by `keccak256(bytes(dataLink))`.
contract SyncContract is Initializable, UUPSUpgradeable, ReentrancyGuardUpgradeable {
    // -------- Errors (mirror Solana) --------
    error ErrCallerNotAdmin();
    error ErrInvalidDataLink();
    error ErrDataLinkEmpty();
    error ErrDataLinkTooLarge();
    error ErrDataAlreadyRated();
    error ErrAgentIsNotEnabled();
    error ErrInvalidRating();
    error ErrPrimaryCategoryTooLarge();
    error ErrSecondaryCategoryTooLarge();
    error ErrSeedMustbeDeletedBeforeRating();
    error ErrAgentNotCreated();
    error ErrAgentAlreadyCreated();
    error ErrAlreadyInitializedSubmission();
    error ErrZeroAddress();

    // -------- Constants (mirror Solana sizes) --------
    uint256 public constant DATA_LINK_SIZE = 256;
    uint256 public constant PRIMARY_CATEGORY_SIZE = 32;
    uint256 public constant SECONDARY_CATEGORY_SIZE = 32;

    // -------- Storage --------
    address public admin;
    IStrovaCreditToken public creditToken;

    struct AgentConfig {
        bool exists;
        bool isEnabled;
    }

    struct AgentResponse {
        address agent;
        bool isValid;
        bool isSeedDeleted;
        bool hasRating;
        uint8 rating;
        uint256 calculatedCredits;
        bool hasSyntheticDataLink;
        bytes syntheticDataLink; // present if hasSyntheticDataLink == true
    }

    struct DataHeader {
        bytes32 primaryCategory;
        bytes32 secondaryCategory;
    }

    struct DataSubmission {
        bool exists;
        address user;
        uint256 timestamp;
        string domain;
        string dataType;
        string dataFormat;
        uint256 fileSizeInKB;
        bytes dataLink; // <= 256 bytes, ASCII
        DataHeader header;
        bool isRated;
        AgentResponse response;
    }

    mapping(address => AgentConfig) public agentConfigs;
    mapping(address => uint256) public accumulatedCredits;
    mapping(bytes32 => DataSubmission) private _submissions;
    // Secondary index: per-user submission keys (append-only, paginated reads).
    mapping(address => uint256) public userSubmissionCount;
    mapping(address => mapping(uint256 => bytes32)) private _userSubmissionKeyAt;

    struct SubmissionSummary {
        bytes32 dataKey;
        address user;
        uint256 timestamp;
        string dataLink;
        string domain;
        string dataType;
        string dataFormat;
        uint256 fileSizeInKB;
        bytes32 primaryCategory;
        bytes32 secondaryCategory;
        bool isRated;
    }

    // -------- Events --------
    event Initialized(address indexed admin, address indexed creditToken);
    event AdminTransferred(address indexed oldAdmin, address indexed newAdmin);
    event AgentCreated(address indexed agent);
    event AgentAllowed(address indexed agent);
    event DataSubmitted(bytes32 indexed dataKey, address indexed user, string dataLink);
    event DataRated(
        bytes32 indexed dataKey,
        address indexed agent,
        address indexed user,
        bool isValid,
        bool hasRating,
        uint8 rating,
        uint256 calculatedCredits,
        bool sendTokensImmediately
    );
    event CreditsClaimed(address indexed user, uint256 credits, uint256 tokenAmount);
    event MintAuthorityTransferred(address indexed newAuthority);

    // -------- Modifiers --------
    modifier onlyAdmin() {
        if (msg.sender != admin) revert ErrCallerNotAdmin();
        _;
    }

    // -------- Initializer / Upgrade auth --------
    function initialize(address admin_, address creditToken_) external initializer {
        __UUPSUpgradeable_init();
        __ReentrancyGuard_init();

        admin = admin_;
        creditToken = IStrovaCreditToken(creditToken_);

        emit Initialized(admin_, creditToken_);
    }

    function _authorizeUpgrade(address) internal view override onlyAdmin {}

    /// @notice Transfer contract admin to a new address.
    function transferAdmin(address newAdmin) external onlyAdmin {
        if (newAdmin == address(0)) revert ErrZeroAddress();
        address old = admin;
        admin = newAdmin;
        emit AdminTransferred(old, newAdmin);
    }

    // -------- Public / External API --------

    /// @notice Create agent config for msg.sender. Mirrors Solana PDA init.
    function createAgent() external {
        AgentConfig storage cfg = agentConfigs[msg.sender];
        if (cfg.exists) revert ErrAgentAlreadyCreated();
        cfg.exists = true;
        cfg.isEnabled = false;
        emit AgentCreated(msg.sender);
    }

    /// @notice Enable an agent. Only admin.
    function allowAgent(address agent) external onlyAdmin {
        AgentConfig storage cfg = agentConfigs[agent];
        if (!cfg.exists) revert ErrAgentNotCreated();
        cfg.isEnabled = true;
        emit AgentAllowed(agent);
    }

    function submitData(
        string calldata dataLink,
        string calldata primaryCategory,
        string calldata secondaryCategory,
        string calldata domain,
        string calldata dataType,
        string calldata dataFormat,
        uint256 fileSizeInKB
    ) external {
        bytes32 dataKey;
        bytes32 primaryCat;
        bytes32 secondaryCat;

        // Scope temps so we don't hit "stack too deep" in non-IR compilation.
        bytes memory linkBytes;
        {
            linkBytes = bytes(dataLink);
            if (linkBytes.length == 0) revert ErrDataLinkEmpty();
            if (linkBytes.length > DATA_LINK_SIZE) revert ErrDataLinkTooLarge();
            if (!_isAscii(linkBytes)) revert ErrInvalidDataLink();
            dataKey = keccak256(linkBytes);
        }

        {
            bytes memory primaryBytes = bytes(primaryCategory);
            if (primaryBytes.length > PRIMARY_CATEGORY_SIZE) revert ErrPrimaryCategoryTooLarge();
            primaryCat = _toBytes32RightPadded(primaryBytes);
        }

        {
            bytes memory secondaryBytes = bytes(secondaryCategory);
            if (secondaryBytes.length > SECONDARY_CATEGORY_SIZE) revert ErrSecondaryCategoryTooLarge();
            secondaryCat = _toBytes32RightPadded(secondaryBytes);
        }

        DataSubmission storage sub = _submissions[dataKey];
        if (sub.exists) revert ErrAlreadyInitializedSubmission();

        sub.exists = true;
        sub.user = msg.sender;
        sub.timestamp = block.timestamp;
        sub.domain = domain;
        sub.dataType = dataType;
        sub.dataFormat = dataFormat;
        sub.fileSizeInKB = fileSizeInKB;
        sub.dataLink = linkBytes;
        sub.header = DataHeader({
            primaryCategory: primaryCat,
            secondaryCategory: secondaryCat
        });
        sub.isRated = false;

        // Index by user for pagination (IPFS hash remains globally unique).
        {
            uint256 idx = userSubmissionCount[msg.sender];
            _userSubmissionKeyAt[msg.sender][idx] = dataKey;
            userSubmissionCount[msg.sender] = idx + 1;
        }

        emit DataSubmitted(dataKey, msg.sender, dataLink);
    }

    /// @notice Rate an existing data submission.
    /// @param dataLink Original data link used on submission.
    /// @param isSeedDeleted Must be true or it reverts.
    /// @param isValid Agent explicitly marks the file valid/invalid.
    /// @param hasRating If true, `rating` must be 0..100 inclusive. If false, rating is ignored.
    /// @param rating Optional rating (only used when hasRating=true).
    /// @param hasSyntheticDataLink Optional synthetic link (validated only if hasSyntheticDataLink=true).
    /// @param syntheticDataLink Synthetic link value when present.
    /// @param sendTokensImmediately If true, mints ERC20 to the user; otherwise accumulates credits.
    function rateData(
        string calldata dataLink,
        bool isSeedDeleted,
        bool isValid,
        bool hasRating,
        uint8 rating,
        bool hasSyntheticDataLink,
        string calldata syntheticDataLink,
        bool sendTokensImmediately
    ) external nonReentrant {
        _requireEnabledAgent(msg.sender);

        bytes32 dataKey = keccak256(bytes(dataLink));
        DataSubmission storage sub = _submissions[dataKey];
        if (!sub.exists) revert ErrInvalidDataLink();
        if (sub.isRated) revert ErrDataAlreadyRated();
        if (hasRating && rating > 100) revert ErrInvalidRating();
        if (!isSeedDeleted) revert ErrSeedMustbeDeletedBeforeRating();

        bytes memory syntheticBytes = _validateOptionalSyntheticLink(hasSyntheticDataLink, syntheticDataLink);

        // Credits are only granted when agent marks valid AND provides a rating.
        uint256 calculatedCredits = (isValid && hasRating) ? uint256(rating) : 0;

        sub.isRated = true;
        _setAgentResponse(
            sub,
            msg.sender,
            isValid,
            isSeedDeleted,
            hasRating,
            rating,
            calculatedCredits,
            hasSyntheticDataLink,
            syntheticBytes
        );

        if (!sendTokensImmediately) {
            accumulatedCredits[sub.user] += calculatedCredits;
        } else {
            _mintCredits(sub.user, calculatedCredits);
        }

        emit DataRated(
            dataKey,
            msg.sender,
            sub.user,
            isValid,
            hasRating,
            rating,
            calculatedCredits,
            sendTokensImmediately
        );
    }

    /// @notice Claim accumulated credits (mints ERC20) and resets to 0.
    /// Mirrors Solana atomicity (revert keeps credits unchanged).
    function claimCredits() external nonReentrant {
        uint256 credits = accumulatedCredits[msg.sender];
        accumulatedCredits[msg.sender] = 0;

        uint256 tokenAmount = _creditsToTokenAmount(credits);
        creditToken.mint(msg.sender, tokenAmount);

        emit CreditsClaimed(msg.sender, credits, tokenAmount);
    }

    /// @notice Transfer mint authority of the credit token away from this contract.
    /// After this, `rateData(..., sendTokensImmediately=true)` and `claimCredits()` will revert.
    function transferMintAuthority(address newAuthority) external onlyAdmin {
        creditToken.transferMintAuthority(newAuthority);
        emit MintAuthorityTransferred(newAuthority);
    }

    // -------- Views --------

    function getSubmission(bytes32 dataKey) external view returns (DataSubmission memory) {
        return _submissions[dataKey];
    }

    function getSubmissionKey(string calldata dataLink) external pure returns (bytes32) {
        return keccak256(bytes(dataLink));
    }

    /// @notice Get the submission key for a user at a specific index (0-based).
    function getUserSubmissionKeyAt(address user, uint256 index) external view returns (bytes32) {
        if (index >= userSubmissionCount[user]) return bytes32(0);
        return _userSubmissionKeyAt[user][index];
    }

    /// @notice Get a page of submission keys for a user.
    function getUserSubmissionKeys(
        address user,
        uint256 start,
        uint256 limit
    ) external view returns (bytes32[] memory keys) {
        uint256 count = userSubmissionCount[user];
        if (start >= count || limit == 0) {
            return new bytes32[](0);
        }
        uint256 end = start + limit;
        if (end > count) end = count;
        uint256 size = end - start;
        keys = new bytes32[](size);
        for (uint256 i = 0; i < size; i++) {
            keys[i] = _userSubmissionKeyAt[user][start + i];
        }
    }

    /// @notice Get a page of submission summaries (metadata) for a user.
    /// This is paginated to keep calls bounded even if a user has many submissions.
    function getUserSubmissionSummaries(
        address user,
        uint256 start,
        uint256 limit
    ) external view returns (SubmissionSummary[] memory out) {
        uint256 count = userSubmissionCount[user];
        if (start >= count || limit == 0) {
            return new SubmissionSummary[](0);
        }
        uint256 end = start + limit;
        if (end > count) end = count;
        uint256 size = end - start;
        out = new SubmissionSummary[](size);
        for (uint256 i = 0; i < size; i++) {
            bytes32 key = _userSubmissionKeyAt[user][start + i];
            DataSubmission storage s = _submissions[key];
            out[i] = SubmissionSummary({
                dataKey: key,
                user: s.user,
                timestamp: s.timestamp,
                dataLink: string(s.dataLink),
                domain: s.domain,
                dataType: s.dataType,
                dataFormat: s.dataFormat,
                fileSizeInKB: s.fileSizeInKB,
                primaryCategory: s.header.primaryCategory,
                secondaryCategory: s.header.secondaryCategory,
                isRated: s.isRated
            });
        }
    }

    // -------- Internals --------

    function _mintCredits(address to, uint256 credits) internal {
        uint256 tokenAmount = _creditsToTokenAmount(credits);
        creditToken.mint(to, tokenAmount);
    }

    function _creditsToTokenAmount(uint256 credits) internal view returns (uint256) {
        uint8 dec = creditToken.decimals();
        return credits * (10 ** uint256(dec));
    }

    function _isAscii(bytes memory b) internal pure returns (bool) {
        uint256 len = b.length;
        for (uint256 i = 0; i < len; i++) {
            if (uint8(b[i]) > 0x7F) return false;
        }
        return true;
    }

    function _requireEnabledAgent(address agent) internal view {
        AgentConfig storage cfg = agentConfigs[agent];
        if (!cfg.isEnabled) revert ErrAgentIsNotEnabled();
    }

    function _validateOptionalSyntheticLink(
        bool hasSyntheticDataLink,
        string calldata syntheticDataLink
    ) internal pure returns (bytes memory syntheticBytes) {
        if (!hasSyntheticDataLink) return "";
        syntheticBytes = bytes(syntheticDataLink);
        if (syntheticBytes.length > DATA_LINK_SIZE) revert ErrDataLinkTooLarge();
        if (!_isAscii(syntheticBytes)) revert ErrInvalidDataLink();
    }

    function _setAgentResponse(
        DataSubmission storage sub,
        address agent,
        bool isValid,
        bool isSeedDeleted,
        bool hasRating,
        uint8 rating,
        uint256 calculatedCredits,
        bool hasSyntheticDataLink,
        bytes memory syntheticBytes
    ) internal {
        AgentResponse storage r = sub.response;
        r.agent = agent;
        r.isValid = isValid;
        r.isSeedDeleted = isSeedDeleted;
        r.hasRating = hasRating;
        r.rating = rating;
        r.calculatedCredits = calculatedCredits;
        r.hasSyntheticDataLink = hasSyntheticDataLink;
        r.syntheticDataLink = syntheticBytes;
    }

    function _toBytes32RightPadded(bytes memory b) internal pure returns (bytes32 out) {
        // right padded with zeros (same as Solana fixed [u8;32] with prefix filled)
        uint256 len = b.length;
        if (len == 0) return bytes32(0);
        assembly {
            out := mload(add(b, 32))
        }
    }
}


